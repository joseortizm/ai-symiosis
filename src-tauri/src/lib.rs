// Module declarations
mod database;
mod search;
#[cfg(test)]
mod tests;

// External crates
use comrak::{markdown_to_html, ComrakOptions};
use database::{get_database_path as get_db_path, get_db_connection};
use rusqlite::{params, Connection};
use search::search_notes_hybrid;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use walkdir::WalkDir;

// Global static configuration
static APP_CONFIG: LazyLock<RwLock<AppConfig>> = LazyLock::new(|| RwLock::new(load_config()));

// Configuration structure and defaults
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    notes_directory: String,
    #[serde(default = "default_max_results")]
    max_search_results: usize,
    #[serde(default = "default_global_shortcut")]
    global_shortcut: String,
    #[serde(default = "default_editor_mode")]
    editor_mode: String,
    #[serde(default = "default_markdown_theme")]
    markdown_theme: String,
}

fn default_max_results() -> usize {
    100
}

fn default_global_shortcut() -> String {
    "Ctrl+Shift+N".to_string()
}

fn default_editor_mode() -> String {
    "basic".to_string()
}

fn default_markdown_theme() -> String {
    "dark_dimmed".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            notes_directory: get_default_notes_dir(),
            max_search_results: default_max_results(),
            global_shortcut: default_global_shortcut(),
            editor_mode: default_editor_mode(),
            markdown_theme: default_markdown_theme(),
        }
    }
}

// Utility functions
fn parse_shortcut(shortcut_str: &str) -> Option<Shortcut> {
    shortcut_str.parse().ok()
}

fn get_default_notes_dir() -> String {
    if let Some(home_dir) = home::home_dir() {
        home_dir
            .join("Documents")
            .join("Notes")
            .to_string_lossy()
            .to_string()
    } else {
        "./notes".to_string()
    }
}

fn get_config_path() -> PathBuf {
    if let Some(home_dir) = home::home_dir() {
        home_dir.join(".symiosis").join("config.toml")
    } else {
        PathBuf::from(".symiosis/config.toml")
    }
}

fn get_config_notes_dir() -> PathBuf {
    let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    PathBuf::from(&config.notes_directory)
}

fn get_database_path() -> PathBuf {
    get_db_path().unwrap_or_else(|_| PathBuf::from("./symiosis/notes.sqlite"))
}

fn validate_note_name(note_name: &str) -> Result<(), String> {
    // Check for empty name
    if note_name.trim().is_empty() {
        return Err("Note name cannot be empty".to_string());
    }
    // Prevent path traversal attacks
    if std::path::Path::new(note_name)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("Path traversal not allowed".to_string());
    }

    if note_name.contains('\\') {
        return Err("Invalid note name".to_string());
    }

    if std::path::Path::new(note_name).is_absolute() {
        return Err("Absolute paths not allowed".to_string());
    }
    // Prevent hidden files and system files
    if note_name.starts_with('.') {
        return Err("Note name cannot start with a dot".to_string());
    }
    // Prevent excessively long names
    if note_name.len() > 255 {
        return Err("Note name too long".to_string());
    }
    Ok(())
}

fn render_note(filename: &str, content: &str) -> String {
    if filename.ends_with(".md") || filename.ends_with(".markdown") {
        markdown_to_html(content, &ComrakOptions::default())
    } else {
        format!("<pre>{}</pre>", html_escape::encode_text(content))
    }
}

// Configuration management
fn load_config() -> AppConfig {
    let config_path = get_config_path();

    match fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str::<AppConfig>(&content) {
            Ok(config) => {
                println!("Loaded config from: {}", config_path.display());
                config
            }
            Err(e) => {
                eprintln!("Failed to parse config file: {e}. Using defaults.");
                AppConfig::default()
            }
        },
        Err(_) => {
            println!("Config file not found, using defaults");
            AppConfig::default()
        }
    }
}

fn reload_config(app_handle: Option<AppHandle>) -> Result<(), String> {
    let new_config = load_config();
    let mut config = APP_CONFIG
        .write()
        .map_err(|e| format!("Failed to write config lock: {}", e))?;
    *config = new_config.clone();

    // Emit config changed event to frontend
    if let Some(app) = app_handle {
        let _ = app.emit("config-changed", &new_config);
    }

    Ok(())
}

// Database operations
fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS notes USING fts5(filename, content, modified UNINDEXED);")
}

fn load_all_notes_into_sqlite(conn: &mut Connection) -> rusqlite::Result<()> {
    let notes_dir = get_config_notes_dir();

    if !notes_dir.exists() {
        if let Err(e) = fs::create_dir_all(&notes_dir) {
            eprintln!("Failed to create notes directory: {}", e);
            return Ok(());
        }
    }

    // Get current files from filesystem
    let mut filesystem_files = std::collections::HashMap::new();

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

            filesystem_files.insert(filename, (path.to_path_buf(), modified));
        }
    }

    // Get current files from database
    let mut database_files = std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT filename, modified FROM notes")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (filename, modified) = row?;
            database_files.insert(filename, modified);
        }
    }

    let tx = conn.transaction()?;

    // Remove files that no longer exist on filesystem
    for filename in database_files.keys() {
        if !filesystem_files.contains_key(filename) {
            tx.execute("DELETE FROM notes WHERE filename = ?1", params![filename])?;
        }
    }

    // Add or update files that are new or modified
    for (filename, (path, fs_modified)) in filesystem_files {
        let db_modified = database_files.get(&filename).copied().unwrap_or(0);

        // Only update if file is new or has been modified
        if fs_modified != db_modified {
            let content = fs::read_to_string(path).unwrap_or_default();

            // Use INSERT OR REPLACE for upsert behavior
            tx.execute(
                "INSERT OR REPLACE INTO notes (filename, content, modified) VALUES (?1, ?2, ?3)",
                params![filename, content, fs_modified],
            )?;
        }
    }

    tx.commit()
}

// Tauri command handlers - Query operations
#[tauri::command]
fn list_all_notes() -> Result<Vec<String>, String> {
    let conn = get_db_connection()?;

    let mut stmt = conn
        .prepare("SELECT filename FROM notes ORDER BY modified DESC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for r in rows {
        if let Ok(filename) = r {
            results.push(filename);
        }
    }

    Ok(results)
}

#[tauri::command]
fn search_notes(query: &str) -> Result<Vec<String>, String> {
    let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    search_notes_hybrid(query, config.max_search_results)
}

#[tauri::command]
fn get_note_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);
    if !note_path.exists() {
        return Err(format!("Note not found: {}", note_name));
    }
    let content = fs::read_to_string(&note_path).map_err(|e| e.to_string())?;
    Ok(render_note(note_name, &content))
}

#[tauri::command]
fn get_note_raw_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);
    if !note_path.exists() {
        return Err(format!("Note not found: {}", note_name));
    }
    let content = fs::read_to_string(&note_path).map_err(|e| e.to_string())?;
    Ok(content)
}

// Tauri command handlers - Mutation operations
#[tauri::command]
fn create_new_note(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)?;

    let note_path = get_config_notes_dir().join(note_name);

    // Check if note already exists
    if note_path.exists() {
        return Err(format!("Note '{}' already exists", note_name));
    }

    // Create parent directories if they don't exist
    if let Some(parent) = note_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Create empty note file
    fs::write(&note_path, "").map_err(|e| format!("Failed to create note: {}", e))?;

    let conn = get_db_connection()?;

    let modified = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO notes (filename, content, modified) VALUES (?1, ?2, ?3)",
        params![note_name, "", modified],
    )
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(())
}

#[tauri::command]
fn save_note(note_name: &str, content: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);

    if let Some(parent) = note_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    fs::write(&note_path, content).map_err(|e| format!("Failed to save note: {}", e))?;

    let conn = get_db_connection()?;

    let modified = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // First try to update existing note
    let updated_rows = conn
        .execute(
            "UPDATE notes SET content = ?2, modified = ?3 WHERE filename = ?1",
            params![note_name, content, modified],
        )
        .map_err(|e| format!("Database error: {}", e))?;

    // If no rows were updated, insert new note
    if updated_rows == 0 {
        conn.execute(
            "INSERT INTO notes (filename, content, modified) VALUES (?1, ?2, ?3)",
            params![note_name, content, modified],
        )
        .map_err(|e| format!("Database error: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
fn rename_note(old_name: String, new_name: String) -> Result<(), String> {
    validate_note_name(&old_name)?;
    validate_note_name(&new_name)?;

    let notes_dir = get_config_notes_dir();
    let old_path = notes_dir.join(&old_name);
    let new_path = notes_dir.join(&new_name);

    if !old_path.exists() {
        return Err(format!("Note '{}' not found", old_name));
    }

    if new_path.exists() {
        return Err(format!("Note '{}' already exists", new_name));
    }

    // Rename the file
    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename note: {}", e))?;

    let conn = get_db_connection()?;

    conn.execute(
        "UPDATE notes SET filename = ?1 WHERE filename = ?2",
        params![new_name, old_name],
    )
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(())
}

#[tauri::command]
fn delete_note(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);

    // Check if note exists
    if !note_path.exists() {
        return Err(format!("Note '{}' not found", note_name));
    }

    // Delete the file
    fs::remove_file(&note_path).map_err(|e| format!("Failed to delete note: {}", e))?;

    let conn = get_db_connection()?;

    conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name])
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(())
}

// Tauri command handlers - System operations
#[tauri::command]
fn refresh_cache(app: AppHandle) -> Result<(), String> {
    // Reload config first
    reload_config(Some(app)).map_err(|e| format!("Failed to reload config: {}", e))?;

    // Then refresh database
    let mut conn = get_db_connection()?;
    init_db(&conn).map_err(|e| format!("Database initialization error: {}", e))?;
    load_all_notes_into_sqlite(&mut conn).map_err(|e| format!("Failed to load notes: {}", e))?;
    Ok(())
}

#[tauri::command]
fn open_note_in_editor(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);
    if !note_path.exists() {
        return Err(format!("Note not found: {}", note_name));
    }

    std::process::Command::new("open")
        // .arg("-a")
        // .arg("TextEdit")
        .arg(&note_path)
        .status()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn open_note_folder(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);
    if !note_path.exists() {
        return Err(format!("Note not found: {}", note_name));
    }

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg("-R")
        .arg(note_path)
        .status()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    {
        let path_str = note_path.to_str().ok_or("Invalid path encoding")?;
        std::process::Command::new("explorer")
            .arg(format!("/select,\"{}\"", path_str)) // Quotes required for spaces
            .status()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        let folder_path = note_path.parent().ok_or("Unable to determine folder")?;
        std::process::Command::new("xdg-open")
            .arg(folder_path)
            .status()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// Tauri command handlers - Configuration operations
#[tauri::command]
fn get_config_content() -> Result<String, String> {
    let config_path = get_config_path();

    match fs::read_to_string(&config_path) {
        Ok(content) => Ok(content),
        Err(_) => {
            let template = include_str!("../config-template.toml")
                .replace("{DEFAULT_NOTES_DIR}", &get_default_notes_dir());
            Ok(template)
        }
    }
}

#[tauri::command]
fn save_config_content(content: &str) -> Result<(), String> {
    let config_path = get_config_path();

    // Validate TOML format before saving
    let _: AppConfig =
        toml::from_str(content).map_err(|e| format!("Invalid TOML format: {}", e))?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    fs::write(&config_path, content).map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

#[tauri::command]
fn config_exists() -> bool {
    get_config_path().exists()
}

#[tauri::command]
fn get_editor_mode() -> String {
    let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    config.editor_mode.clone()
}

#[tauri::command]
fn get_markdown_theme() -> String {
    let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    config.markdown_theme.clone()
}

// Tauri command handlers - Window operations
#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    match app.get_webview_window("main") {
        Some(window) => {
            window
                .show()
                .map_err(|e| format!("Failed to show window: {}", e))?;
            window
                .set_focus()
                .map_err(|e| format!("Failed to focus window: {}", e))?;
        }
        None => {
            // Create the window if it doesn't exist
            let _window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
                .title("Symiosis Notes")
                .inner_size(1200.0, 800.0)
                .center()
                .visible(false) // Let window-state plugin handle visibility to prevent flash
                .build()
                .map_err(|e| format!("Failed to create window: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
fn hide_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|e| format!("Failed to hide window: {}", e))?;
    }
    Ok(())
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

// Initialization functions
pub fn initialize_notes() {
    let db_path = get_database_path();
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut conn = match get_db_connection() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to open database: {}. Application will continue with limited functionality.", e);
            return;
        }
    };

    if let Err(e) = init_db(&conn) {
        eprintln!("Failed to initialize database: {}. Application will continue with limited functionality.", e);
        return;
    }

    if !get_config_path().exists() {
        if let Err(e) = conn.execute("DELETE FROM notes", []) {
            eprintln!("Failed to purge database: {}. Continuing anyway.", e);
        }
        return;
    }

    if let Err(e) = load_all_notes_into_sqlite(&mut conn) {
        eprintln!(
            "Failed to load notes into database: {}. Some notes may not be searchable.",
            e
        );
    }
}

// Main application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    initialize_notes();

    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // Setup the system tray
            setup_tray(app.handle())?;

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
            create_new_note,
            delete_note,
            rename_note,
            save_note,
            refresh_cache,
            open_note_in_editor,
            open_note_folder,
            list_all_notes,
            show_main_window,
            hide_main_window,
            get_config_content,
            save_config_content,
            config_exists,
            get_editor_mode,
            get_markdown_theme
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
