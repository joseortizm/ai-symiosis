// Module declarations
mod commands;
mod config;
mod database;
mod search;
mod services;
#[cfg(test)]
mod tests;
mod watcher;

// External crates
use commands::*;
use config::{
    get_available_themes, get_config_path, get_editor_config, get_general_config,
    get_interface_config, get_preferences_config, get_shortcuts_config, load_config,
    parse_shortcut, save_config_content, AppConfig,
};
use database::{get_database_path as get_db_path, get_db_connection, with_db};
use rusqlite::{params, Connection};
use services::note_service;
use std::fs;
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, LazyLock, Mutex, RwLock};
use std::time::UNIX_EPOCH;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use walkdir::WalkDir;
use watcher::setup_notes_watcher;

// Global static configuration
pub static APP_CONFIG: LazyLock<RwLock<AppConfig>> = LazyLock::new(|| RwLock::new(load_config()));

pub static WAS_FIRST_RUN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// Global database lock to prevent concurrent database operations
static DB_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

// Global flag to prevent file watcher from triggering cache refresh during programmatic operations
pub static PROGRAMMATIC_OPERATION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

// Number of most recent notes to get immediate HTML rendering during startup
// Remaining notes get metadata-only and are processed on demand
const IMMEDIATE_RENDER_COUNT: usize = 2000;

pub fn get_config_notes_dir() -> PathBuf {
    let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    PathBuf::from(&config.notes_directory)
}

// Database operations
pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS notes USING fts5(filename, content, html_render, modified UNINDEXED, is_indexed UNINDEXED);")?;

    // Check for discrepancy by looking for duplicate filenames
    let mut stmt = conn.prepare(
        "SELECT filename, COUNT(*) as count FROM notes GROUP BY filename HAVING count > 1",
    )?;
    let duplicate_rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
    })?;

    let duplicates: Result<Vec<_>, _> = duplicate_rows.collect();
    if let Ok(dups) = duplicates {
        if !dups.is_empty() {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT),
                Some(format!(
                    "Database discrepancy detected: {} files have duplicate entries",
                    dups.len()
                )),
            ));
        }
    }

    Ok(())
}

pub fn recreate_database() -> Result<(), String> {
    eprintln!("Database discrepancy detected. Recreating database tables...");

    let mut conn = get_db_connection()?;

    // Drop the existing table and recreate it
    conn.execute("DROP TABLE IF EXISTS notes", [])
        .map_err(|e| format!("Failed to drop corrupted table: {}", e))?;

    init_db(&conn).map_err(|e| format!("Failed to initialize fresh database: {}", e))?;

    eprintln!("Fresh database table created. Performing full sync from filesystem...");

    // Perform a complete sync from filesystem
    load_all_notes_into_sqlite(&mut conn)
        .map_err(|e| format!("Failed to populate fresh database: {}", e))?;

    eprintln!("Database recovery completed successfully.");
    Ok(())
}

pub async fn recreate_database_with_progress(
    app_handle: &AppHandle,
    reason: &str,
) -> Result<(), String> {
    let _ = app_handle.emit("db-loading-progress", "Rebuilding notes database...");
    eprintln!("{}", reason);

    let mut conn = get_db_connection()?;

    // Drop the existing table and recreate it
    conn.execute("DROP TABLE IF EXISTS notes", [])
        .map_err(|e| format!("Failed to drop corrupted table: {}", e))?;

    init_db(&conn).map_err(|e| format!("Failed to initialize fresh database: {}", e))?;

    let _ = app_handle.emit("db-loading-progress", "Rendering notes...");
    eprintln!("Fresh database table created. Performing full sync from filesystem...");

    // Perform a complete sync from filesystem
    let result = tokio::task::spawn_blocking(move || load_all_notes_into_sqlite(&mut conn))
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

    result.map_err(|e| format!("Failed to populate fresh database: {}", e))?;

    let _ = app_handle.emit("db-loading-progress", "Notes database ready.");
    eprintln!("Database rebuild completed successfully.");
    Ok(())
}

pub fn load_all_notes_into_sqlite(conn: &mut Connection) -> rusqlite::Result<()> {
    load_all_notes_into_sqlite_with_progress(conn, None)
}

pub fn load_all_notes_into_sqlite_with_progress(
    conn: &mut Connection,
    app_handle: Option<&AppHandle>,
) -> rusqlite::Result<()> {
    // Prevent concurrent database operations to avoid FTS5 race conditions
    let _lock = DB_LOCK.lock().unwrap();

    let notes_dir = get_config_notes_dir();

    if !notes_dir.exists() {
        if let Err(e) = fs::create_dir_all(&notes_dir) {
            eprintln!("Failed to create notes directory: {}", e);
            return Ok(());
        }
    }

    // Get current files from filesystem, sorted by modification time (newest first)
    let mut filesystem_files = Vec::new();

    for entry in WalkDir::new(&notes_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            let relative = path.strip_prefix(&notes_dir).unwrap_or(path);
            let filename = relative.to_string_lossy().to_string();

            if filename.contains("/.") || filename.starts_with('.') {
                continue;
            }

            let modified = entry
                .path()
                .metadata()
                .and_then(|m| m.modified())
                .map(|mtime| {
                    mtime
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            filesystem_files.push((filename, path.to_path_buf(), modified));
        }
    }

    // Sort by modification time, newest first
    filesystem_files.sort_by(|a, b| b.2.cmp(&a.2));

    // Get current files from database
    let mut database_files = std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT filename, modified, is_indexed FROM notes")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, bool>(2).unwrap_or(false),
            ))
        })?;

        for row in rows {
            let (filename, modified, is_indexed) = row?;
            database_files.insert(filename, (modified, is_indexed));
        }
    }

    let tx = conn.transaction()?;

    // Remove files that no longer exist on filesystem
    let filesystem_filenames: std::collections::HashSet<_> =
        filesystem_files.iter().map(|(name, _, _)| name).collect();
    for filename in database_files.keys() {
        if !filesystem_filenames.contains(filename) {
            tx.execute("DELETE FROM notes WHERE filename = ?1", params![filename])?;
        }
    }

    let total_files = filesystem_files.len();

    // Process files with progress reporting
    for (index, (filename, path, fs_modified)) in filesystem_files.iter().enumerate() {
        // Emit progress update every 10 files or on first/last file
        if let Some(app) = app_handle {
            if index == 0 || (index + 1) % 10 == 0 || index == total_files - 1 {
                let progress_msg = format!("Loading {} of {} notes...", index + 1, total_files);
                let _ = app.emit("db-loading-progress", progress_msg);
            }
        }

        let (db_modified, is_indexed) = database_files.get(filename).copied().unwrap_or((0, false));

        // Only update if file is new or has been modified
        if *fs_modified != db_modified {
            let content = fs::read_to_string(path).unwrap_or_default();

            if index < IMMEDIATE_RENDER_COUNT {
                // First 300 files: full processing with HTML render
                let html_render = note_service::render_note(filename, &content);
                tx.execute(
                    "INSERT OR REPLACE INTO notes (filename, content, html_render, modified, is_indexed) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![filename, content, html_render, *fs_modified, true],
                )?;
            } else {
                // Remaining files: metadata only, defer HTML rendering
                tx.execute(
                    "INSERT OR REPLACE INTO notes (filename, content, html_render, modified, is_indexed) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![filename, content, "", *fs_modified, false],
                )?;
            }
        } else if !is_indexed && index < IMMEDIATE_RENDER_COUNT {
            // File hasn't changed but needs indexing (upgrade existing entry)
            let content = fs::read_to_string(path).unwrap_or_default();
            let html_render = note_service::render_note(filename, &content);
            tx.execute(
                "UPDATE notes SET html_render = ?2, is_indexed = ?3 WHERE filename = ?1",
                params![filename, html_render, true],
            )?;
        }
    }

    tx.commit()
}

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
                let _ = show_main_window(app.app_handle().clone());
            }
            "refresh" => {
                let _ = refresh_cache(app.app_handle().clone());
            }
            "settings" => {
                let app_handle = app.app_handle().clone();
                let _ = show_main_window(app_handle.clone());
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
                            let _ = show_main_window(app.clone());
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

// Database integrity checking
pub fn quick_filesystem_sync_check() -> Result<bool, String> {
    let notes_dir = get_config_notes_dir();

    // Skip check if notes directory doesn't exist (new user)
    if !notes_dir.exists() {
        return Ok(true);
    }

    with_db(|conn| {
        // Get up to 100 most recently modified files (matching main app's file filtering)
        let mut files: Vec<_> = WalkDir::new(&notes_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                let path = e.path();
                let relative = path.strip_prefix(&notes_dir).unwrap_or(path);
                let filename = relative.to_string_lossy().to_string();

                // Skip hidden files/folders (same logic as main app)
                if filename.contains("/.") || filename.starts_with('.') {
                    return false;
                }

                // Only include .md files
                path.extension().map_or(false, |ext| ext == "md")
            })
            .collect();

        // Skip check if no files found
        if files.is_empty() {
            return Ok(true);
        }

        // Sort by modification time (most recent first) and take up to 100
        files.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));
        files.truncate(100);

        // Check each file against database
        for entry in files {
            let file_path = entry.path();
            let relative_path = file_path
                .strip_prefix(&notes_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;
            let filename = relative_path.to_string_lossy().to_string();

            // Try to read file content (skip on permission issues with warning)
            let file_content = match std::fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(_) => {
                    eprintln!(
                        "Warning: Could not read file {} during sync check",
                        filename
                    );
                    continue;
                }
            };

            // Get modification time
            let file_modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            // Check if file exists in database with matching content
            let db_result: Result<(String, i64), rusqlite::Error> = conn.query_row(
                "SELECT content, modified FROM notes WHERE filename = ?1",
                params![filename],
                |row| Ok((row.get(0)?, row.get(1)?)),
            );

            match db_result {
                Ok((db_content, db_modified)) => {
                    // Check content match
                    if db_content != file_content {
                        return Ok(false);
                    }
                    // Check modification time match (allow 1 second tolerance)
                    if (db_modified - file_modified).abs() > 1 {
                        return Ok(false);
                    }
                }
                Err(_) => {
                    // File exists in filesystem but not in database
                    return Ok(false);
                }
            }
        }

        Ok(true)
    })
}

// Initialization functions
pub fn initialize_notes() {
    if let Ok(db_path) = get_db_path() {
        if let Some(parent) = db_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
    }

    // Clean up any leftover temp files from previous runs
    let _ = note_service::cleanup_temp_files();

    let conn = match get_db_connection() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to open database: {}. Application will continue with limited functionality.", e);
            return;
        }
    };

    if let Err(e) = init_db(&conn) {
        eprintln!("❌ CRITICAL: Database initialization failed: {}", e);
        eprintln!("🔄 Attempting automatic database recovery...");

        // Attempt automatic recovery by recreating the database
        if let Err(recovery_error) = recreate_database() {
            eprintln!("💥 FATAL: Database recovery failed: {}. Application will continue with limited functionality.", recovery_error);
            return;
        } else {
            eprintln!("✅ Database successfully recovered!");
        }
    } else {
        // Database initialized successfully, perform filesystem sync check
        match quick_filesystem_sync_check() {
            Ok(true) => {
                // Database and filesystem are in sync
            }
            Ok(false) => {
                eprintln!("🔄 Database-filesystem mismatch detected. Rebuilding database...");
                if let Err(e) = recreate_database() {
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
        if let Err(e) = conn.execute("DELETE FROM notes", []) {
            eprintln!("Failed to purge database: {}. Continuing anyway.", e);
        }
    }

    // Note: Notes loading is now deferred to async initialization command
    // This allows the UI to render first before blocking on note loading
}

// Main application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    initialize_notes();

    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(std::sync::RwLock::new(load_config()))
        .setup(|app| {
            // Setup the system tray
            setup_tray(app.handle())?;

            // Setup file system watcher for notes directory
            setup_notes_watcher(app.handle().clone())?;

            // Setup global shortcuts
            #[cfg(desktop)]
            {
                // Get main shortcut from config
                let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
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
                                            let _ = show_main_window(app_handle);
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
            get_note_raw_content,
            get_note_html_content,
            create_new_note,
            delete_note,
            rename_note,
            save_note,
            initialize_notes_with_progress,
            refresh_cache,
            open_note_in_editor,
            open_note_folder,
            list_all_notes,
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
            get_available_themes
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
