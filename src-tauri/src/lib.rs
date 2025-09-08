// Module declarations
mod commands;
mod config;
mod core;
mod database;
mod logging;
mod search;
mod services;
#[cfg(test)]
mod tests;
mod watcher;

// External crates
use commands::*;
use config::{
    get_config_path, get_editor_config, get_general_config, get_interface_config,
    get_preferences_config, get_shortcuts_config, load_config, parse_shortcut, save_config_content,
    scan_available_themes,
};
use core::state::{get_was_first_run, set_global_state, with_config, AppState};
use database::{get_database_path as get_db_path, with_db};
use logging::log;
use services::{database_service, note_service};
use std::fs;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use watcher::setup_notes_watcher;

// MIGRATION: Moved APP_CONFIG out of scope to find all usages via compiler errors
// pub static APP_CONFIG: LazyLock<RwLock<AppConfig>> = LazyLock::new(|| RwLock::new(load_config()));

// These global flags are now managed through AppState and accessed via helper functions
// in core::state - no more global statics needed

// Database operations

// System tray setup
fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    // Create menu items
    let open_item = MenuItem::with_id(app, "open", "Open Symiosis", true, None::<&str>)?;
    let refresh_item =
        MenuItem::with_id(app, "refresh", "Refresh Notes Cache", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Create the menu
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

    // Build the tray icon with icon specified
    let mut tray_builder = TrayIconBuilder::with_id("main-tray");

    // Try to use the default app icon, but continue without icon if it fails
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
                let _ = refresh_cache(app.app_handle().clone());
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

// Database integrity checking

// Initialization functions
pub fn initialize_notes() {
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

    // Clean up any leftover temp files from previous runs
    if let Err(e) = note_service::cleanup_temp_files() {
        log(
            "INIT_CLEANUP",
            "Failed to clean up temp files during initialization",
            Some(&e.to_string()),
        );
    }

    let init_result = with_db(|conn| database_service::init_db(conn).map_err(|e| e.into()));

    if let Err(e) = init_result {
        eprintln!("❌ CRITICAL: Database initialization failed: {}", e);
        eprintln!("🔄 Attempting automatic database recovery...");

        // Attempt automatic recovery by recreating the database
        if let Err(recovery_error) = database_service::recreate_database() {
            eprintln!("💥 FATAL: Database recovery failed: {}. Application will continue with limited functionality.", recovery_error);
            return;
        } else {
            eprintln!("✅ Database successfully recovered!");
        }
    } else {
        // Database initialized successfully, perform filesystem sync check
        match database_service::quick_filesystem_sync_check() {
            Ok(true) => {
                // Database and filesystem are in sync
            }
            Ok(false) => {
                eprintln!("🔄 Database-filesystem mismatch detected. Rebuilding database...");
                if let Err(e) = database_service::recreate_database() {
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
        if let Err(e) = with_db(|conn| conn.execute("DELETE FROM notes", []).map_err(|e| e.into()))
        {
            eprintln!("Failed to purge database: {}. Continuing anyway.", e);
        }
    }

    // Note: Notes loading is now deferred to async initialization command
    // This allows the UI to render first before blocking on note loading
}

// Main application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Early config load with proper state sharing
    let config = load_config();
    let app_state = AppState::new(config);
    set_global_state(std::sync::Arc::new(app_state.clone()));

    // Now initialize_notes() can access config through global state
    initialize_notes();

    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(app_state) // Use the same AppState instance
        .setup(|app| {
            // Global state already set up above

            // Setup the system tray
            setup_tray(app.handle())?;

            // Apply always_on_top setting to main window
            if let Some(window) = app.get_webview_window("main") {
                with_config(|config| {
                    let _ = window.set_always_on_top(config.interface.always_on_top);
                });
            }

            // Setup file system watcher for notes directory
            setup_notes_watcher(app.handle().clone())?;

            // Emit first-run event if this is the first time the app is launched
            if get_was_first_run() {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    let _ = app_handle.emit("first-run-detected", ());
                });
            }

            // Setup global shortcuts
            #[cfg(desktop)]
            {
                // Get main shortcut from config
                let main_shortcut = with_config(|config| {
                    parse_shortcut(&config.global_shortcut).unwrap_or_else(|| {
                        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyN)
                    })
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

    // Hide from dock on macOS
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
