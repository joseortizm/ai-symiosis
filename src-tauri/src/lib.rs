mod commands;
mod config;
mod core;
mod database;
mod logging;
mod search;
mod services;
#[cfg(test)]
mod tests;
mod utilities;
mod watcher;

use commands::*;
use config::{load_config_with_first_run_info, parse_shortcut};
use core::state::AppState;
use logging::log;
use services::database_service;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use watcher::setup_notes_watcher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = load_config_and_initialize_state();

    let app = build_tauri_app_with_plugins(app_state)
        .setup(setup_app_components)
        .on_window_event(handle_window_events)
        .invoke_handler(register_command_handlers())
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| {
            handle_app_build_error(e);
        });

    run_app_with_platform_config(app);
}

pub fn initialize_notes(app_state: &AppState) {
    if let Err(e) = database_service::initialize_application_database(app_state) {
        log(
            "DATABASE_INIT",
            "Application database initialization failed",
            Some(&e.to_string()),
        );
    }
}

fn load_config_and_initialize_state() -> AppState {
    let (config, was_first_run) = load_config_with_first_run_info();
    let app_state = match AppState::new_with_fallback(config) {
        Ok(state) => state,
        Err(e) => {
            log(
                "FATAL_DATABASE_ERROR",
                "Database initialization failed and could not be recovered",
                Some(&e.to_string()),
            );
            log(
                "SHUTDOWN",
                "Application shutting down due to unrecoverable database error",
                None,
            );
            std::process::exit(1);
        }
    };

    if was_first_run {
        app_state.set_first_run(true);
    }

    initialize_notes(&app_state);
    app_state
}

fn build_tauri_app_with_plugins(app_state: AppState) -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(app_state)
}

fn setup_window_configuration(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        if let Some(app_state) = app.try_state::<AppState>() {
            let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
            let _ = window.set_always_on_top(config.interface.always_on_top);
        }
    }
    Ok(())
}

fn setup_notes_watcher_for_app(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(app_state) = app.try_state::<AppState>() {
        setup_notes_watcher(app.handle().clone(), Arc::new(app_state.inner().clone()))?;
    }
    Ok(())
}

fn handle_first_run_detection(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(app_state) = app.try_state::<AppState>() {
        if app_state
            .was_first_run()
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let _ = app_handle.emit("first-run-detected", ());
            });
        }
    }
    Ok(())
}

fn handle_main_window_toggle(app_handle: tauri::AppHandle) {
    match app_handle.get_webview_window("main") {
        Some(window) => {
            if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) {
                let _ = window.hide();
            } else if window.is_visible().unwrap_or(false) && !window.is_focused().unwrap_or(false)
            {
                let _ = window.set_focus();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        None => {
            if let Some(app_state) = app_handle.try_state::<AppState>() {
                let _ = show_main_window(app_handle.clone(), app_state);
            }
        }
    }
}

fn setup_global_shortcuts(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    {
        let config = if let Some(app_state) = app.try_state::<AppState>() {
            app_state
                .config
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .clone()
        } else {
            crate::config::AppConfig::default()
        };
        let main_shortcut = parse_shortcut(&config.global_shortcut).unwrap_or_else(|| {
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyN)
        });

        app.handle()
            .plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        if event.state() == ShortcutState::Pressed {
                            if shortcut == &main_shortcut {
                                let app_handle = app.clone();
                                handle_main_window_toggle(app_handle);
                            }
                        }
                    })
                    .build(),
            )
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        app.global_shortcut()
            .register(main_shortcut)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    }
    Ok(())
}

fn setup_app_components(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    setup_tray(app.handle())?;
    setup_window_configuration(app)?;
    setup_notes_watcher_for_app(app)?;
    handle_first_run_detection(app)?;
    setup_global_shortcuts(app)?;
    Ok(())
}

fn handle_window_events(window: &tauri::Window, event: &tauri::WindowEvent) {
    match event {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            if let Err(e) = window.hide() {
                log(
                    "WINDOW_OPERATION",
                    "Failed to hide window. Continuing anyway.",
                    Some(&e.to_string()),
                );
            }
            api.prevent_close();
        }
        _ => {}
    }
}

fn register_command_handlers(
) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        search_notes,
        get_note_content,
        get_note_html_content,
        create_new_note,
        delete_note,
        rename_note,
        save_note_with_content_check,
        initialize_notes_with_progress,
        refresh_cache,
        open_note_in_editor,
        open_note_folder,
        list_all_notes,
        get_note_versions,
        get_version_content,
        recover_note_version,
        get_deleted_files,
        recover_deleted_file,
        show_main_window,
        hide_main_window,
        get_config_content,
        save_config_content,
        config_exists,
        get_general_config,
        get_interface_config,
        get_editor_config,
        get_shortcuts_config,
        get_preferences_config,
        scan_available_themes
    ]
}

fn handle_app_build_error(e: tauri::Error) -> ! {
    log(
        "APPLICATION_STARTUP",
        "Failed to build Tauri application",
        Some(&e.to_string()),
    );
    std::process::exit(1);
}

fn run_app_with_platform_config(mut app: tauri::App) {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}

fn handle_tray_menu_event(app: &tauri::AppHandle, event: &tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        "open" => {
            let app_handle = app.app_handle().clone();
            if let Some(app_state) = app_handle.try_state::<AppState>() {
                let _ = show_main_window(app_handle.clone(), app_state);
            }
        }
        "refresh" => {
            let app_handle = app.app_handle().clone();
            if let Some(app_state) = app_handle.try_state::<AppState>() {
                let _ = refresh_cache(app_handle.clone(), app_state);
            }
        }
        "settings" => {
            let app_handle = app.app_handle().clone();
            if let Some(app_state) = app_handle.try_state::<AppState>() {
                let _ = show_main_window(app_handle.clone(), app_state);
            }
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.emit("open-preferences", ());
            }
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}

fn handle_tray_icon_event(tray: &tauri::tray::TrayIcon, event: &tauri::tray::TrayIconEvent) {
    if let TrayIconEvent::Click {
        button,
        button_state,
        ..
    } = event
    {
        if *button == tauri::tray::MouseButton::Left
            && *button_state == tauri::tray::MouseButtonState::Up
        {
            let app = tray.app_handle();
            match app.get_webview_window("main") {
                Some(window) => {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                None => {
                    if let Some(app_state) = app.try_state::<AppState>() {
                        let _ = show_main_window(app.clone(), app_state);
                    }
                }
            }
        }
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "Open Symiosis", true, None::<&str>)?;
    let refresh_item =
        MenuItem::with_id(app, "refresh", "Refresh Notes Cache", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &open_item,
            &separator,
            &refresh_item,
            &settings_item,
            &separator,
            &quit_item,
        ],
    )?;

    let mut tray_builder = TrayIconBuilder::with_id("main-tray");

    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    } else {
        log(
            "TRAY_SETUP",
            "Warning: Could not load default window icon for tray. Tray will appear without icon.",
            None,
        );
    }

    let _tray = tray_builder
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| handle_tray_menu_event(app, &event))
        .on_tray_icon_event(|tray, event| handle_tray_icon_event(tray, &event))
        .build(app)?;

    Ok(())
}
