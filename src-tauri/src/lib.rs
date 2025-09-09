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
use database::{get_database_path as get_db_path, with_db};
use logging::log;
use services::{database_service, note_service};
use std::fs;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use utilities::paths::get_config_path;
use watcher::setup_notes_watcher;

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
        eprintln!(
            "Warning: Could not load default window icon for tray. Tray will appear without icon."
        );
    }

    let _tray = tray_builder
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
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
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if button == tauri::tray::MouseButton::Left
                    && button_state == tauri::tray::MouseButtonState::Up
                {
                    // Toggle window visibility on left click
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
        })
        .build(app)?;

    Ok(())
}

pub fn initialize_notes(app_state: &AppState) {
    if let Ok(db_path) = get_db_path() {
        if let Some(parent) = db_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                log(
                    "INIT_ERROR",
                    &format!("Failed to create database directory: {:?}", parent),
                    Some(&e.to_string()),
                );
            }
        }
    }

    if let Err(e) = note_service::cleanup_temp_files() {
        log(
            "INIT_CLEANUP",
            "Failed to clean up temp files during initialization",
            Some(&e.to_string()),
        );
    }

    let init_result = with_db(app_state, |conn| {
        database_service::init_db(conn).map_err(|e| e.into())
    });

    if let Err(e) = init_result {
        let db_path = get_db_path().unwrap_or_default();
        let is_new_db = !db_path.exists();

        if is_new_db {
            eprintln!("🔧 Creating new database...");
        } else {
            eprintln!("❌ CRITICAL: Database initialization failed: {}", e);
            eprintln!("🔄 Attempting automatic database recovery...");
        }

        if let Err(recovery_error) = database_service::recreate_database(app_state) {
            if is_new_db {
                eprintln!("💥 FATAL: Failed to create new database: {}. Application will continue with limited functionality.", recovery_error);
            } else {
                eprintln!("💥 FATAL: Database recovery failed: {}. Application will continue with limited functionality.", recovery_error);
            }
            return;
        } else {
            if is_new_db {
                eprintln!("✅ New database created successfully!");
            } else {
                eprintln!("✅ Database successfully recovered!");
            }
        }
    } else {
        match database_service::quick_filesystem_sync_check(app_state) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("🔄 Database-filesystem mismatch detected. Rebuilding database...");
                if let Err(e) = database_service::recreate_database(app_state) {
                    eprintln!("💥 FATAL: Database rebuild failed: {}. Application will continue with limited functionality.", e);
                    return;
                } else {
                    eprintln!("✅ Database successfully rebuilt from filesystem!");
                }
            }
            Err(e) => {
                eprintln!(
                    "⚠️  Filesystem sync check failed: {}. Continuing without rebuild.",
                    e
                );
            }
        }
    }

    if !get_config_path().exists() {
        if let Err(e) = with_db(app_state, |conn| {
            conn.execute("DELETE FROM notes", []).map_err(|e| e.into())
        }) {
            eprintln!("Failed to purge database: {}. Continuing anyway.", e);
        }
    }

    // Note: Notes loading is now deferred to async initialization command
    // This allows the UI to render first before blocking on note loading
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (config, was_first_run) = load_config_with_first_run_info();
    let app_state = AppState::new_with_fallback(config);
    if was_first_run {
        app_state.set_first_run(true);
    }

    initialize_notes(&app_state);

    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(app_state)
        .setup(|app| {
            setup_tray(app.handle())?;

            if let Some(window) = app.get_webview_window("main") {
                if let Some(app_state) = app.try_state::<AppState>() {
                    let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
                    let _ = window.set_always_on_top(config.interface.always_on_top);
                }
            }

            if let Some(app_state) = app.try_state::<AppState>() {
                setup_notes_watcher(app.handle().clone(), Arc::new(app_state.inner().clone()))?;
            }

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

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |app, shortcut, event| {
                            if event.state() == ShortcutState::Pressed {
                                if shortcut == &main_shortcut {
                                    let app_handle = app.clone();
                                    match app_handle.get_webview_window("main") {
                                        Some(window) => {
                                            if window.is_visible().unwrap_or(false)
                                                && window.is_focused().unwrap_or(false)
                                            {
                                                let _ = window.hide();
                                            } else if window.is_visible().unwrap_or(false)
                                                && !window.is_focused().unwrap_or(false)
                                            {
                                                let _ = window.set_focus();
                                            } else {
                                                let _ = window.show();
                                                let _ = window.set_focus();
                                            }
                                        }
                                        None => {
                                            if let Some(app_state) =
                                                app_handle.try_state::<AppState>()
                                            {
                                                let _ =
                                                    show_main_window(app_handle.clone(), app_state);
                                            }
                                        }
                                    }
                                }
                            }
                        })
                        .build(),
                )?;

                app.global_shortcut().register(main_shortcut)?;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Hide window instead of closing when user clicks X
                    if let Err(e) = window.hide() {
                        eprintln!("Failed to hide window: {}. Continuing anyway.", e);
                    }
                    api.prevent_close();
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
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
        ])
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Failed to build Tauri application: {}", e);
            std::process::exit(1);
        });

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
