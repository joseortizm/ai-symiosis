// Module declarations
mod config;
mod database;
mod search;
#[cfg(test)]
mod tests;
mod watcher;

// External crates
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};
use config::{
    generate_config_template, get_available_themes, get_config_path, get_editor_config,
    get_general_config, get_interface_config, get_preferences_config, get_shortcuts_config,
    load_config, parse_shortcut, reload_config, save_config_content, AppConfig,
};
use database::{
    get_backup_dir_for_notes_path, get_database_path as get_db_path, get_db_connection,
    get_temp_dir,
};
use rusqlite::{params, Connection};
use search::search_notes_hybrid;
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use walkdir::WalkDir;
use watcher::setup_notes_watcher;

// Global static configuration
static APP_CONFIG: LazyLock<RwLock<AppConfig>> = LazyLock::new(|| RwLock::new(load_config()));

// Global database lock to prevent concurrent database operations
static DB_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

// Track last used theme to detect changes
static LAST_RENDER_THEME: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));

fn get_config_notes_dir() -> PathBuf {
    let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    PathBuf::from(&config.notes_directory)
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
        // Get theme from config with fallback
        let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
        let theme = &config.interface.md_render_code_theme;

        // TODO: Add support for loading custom .tmTheme files in the future
        // Validate theme exists, fallback to default if invalid
        let valid_themes = [
            "base16-ocean.dark",
            "base16-eighties.dark",
            "base16-mocha.dark",
            "base16-ocean.light",
            "InspiredGitHub",
            "Solarized (dark)",
            "Solarized (light)",
        ];

        let selected_theme = if valid_themes.contains(&theme.as_str()) {
            theme.as_str()
        } else {
            "base16-ocean.dark" // fallback
        };

        // Create syntax highlighter adapter
        let adapter = comrak::plugins::syntect::SyntectAdapter::new(Some(selected_theme));

        // Configure plugins
        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&adapter);

        // Configure options
        let mut options = ComrakOptions::default();
        options.render.unsafe_ = true; // Allow HTML in markdown

        markdown_to_html_with_plugins(content, &options, &plugins)
    } else {
        format!("<pre>{}</pre>", html_escape::encode_text(content))
    }
}

// Atomic file operations with backup support
fn safe_write_note(note_path: &PathBuf, content: &str) -> Result<(), String> {
    // 1. Create backup in app data directory (preserving relative path structure)
    if note_path.exists() {
        let backup_path = safe_backup_path(note_path)?.with_extension("md.bak");

        // Ensure backup directory structure exists
        if let Some(backup_parent) = backup_path.parent() {
            fs::create_dir_all(backup_parent)
                .map_err(|e| format!("Failed to create backup directory: {}", e))?;
        }

        // Copy existing file to backup
        fs::copy(note_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;
    }

    // 2. Create temp file in app data directory
    let temp_dir = get_temp_dir()?;
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Generate unique temp filename using timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_path = temp_dir.join(format!("write_temp_{}.md", timestamp));

    // 3. Write content to temp file
    fs::write(&temp_path, content).map_err(|e| format!("Failed to write temp file: {}", e))?;

    // 4. Atomic rename to final location
    fs::rename(&temp_path, note_path)
        .map_err(|e| format!("Failed to move temp file to final location: {}", e))?;

    Ok(())
}

fn cleanup_temp_files() -> Result<(), String> {
    let temp_dir = get_temp_dir()?;
    if temp_dir.exists() {
        if let Ok(entries) = fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("write_temp_")
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    Ok(())
}

fn safe_backup_path(note_path: &PathBuf) -> Result<PathBuf, String> {
    let notes_dir = get_config_notes_dir();
    let backup_dir = get_backup_dir_for_notes_path(&notes_dir)?;

    // Get relative path from notes directory to preserve folder structure
    let relative_path = note_path.strip_prefix(&notes_dir).map_err(|_| {
        format!(
            "Note path '{}' is not within configured notes directory '{}'",
            note_path.display(),
            notes_dir.display()
        )
    })?;

    Ok(backup_dir.join(relative_path))
}

// Database operations
fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS notes USING fts5(filename, content, html_render, modified UNINDEXED);")?;

    // Check for corruption by looking for duplicate filenames
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
                    "Database corruption detected: {} files have duplicate entries",
                    dups.len()
                )),
            ));
        }
    }

    Ok(())
}

fn recreate_database() -> Result<(), String> {
    eprintln!("Database corruption detected. Recreating database tables...");

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

async fn recreate_database_with_progress(
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

    let _ = app_handle.emit("db-loading-progress", "Loading notes...");
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

fn load_all_notes_into_sqlite(conn: &mut Connection) -> rusqlite::Result<()> {
    // Prevent concurrent database operations to avoid FTS5 race conditions
    let _lock = DB_LOCK.lock().unwrap();

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
            let content = fs::read_to_string(&path).unwrap_or_default();

            // Generate HTML render from content
            let html_render = render_note(&filename, &content);

            // EXPLICIT DELETE before insert to avoid FTS5 quirks with INSERT OR REPLACE
            tx.execute("DELETE FROM notes WHERE filename = ?1", params![filename])?;
            tx.execute(
                "INSERT INTO notes (filename, content, html_render, modified) VALUES (?1, ?2, ?3, ?4)",
                params![filename, content, html_render, fs_modified],
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
    search_notes_hybrid(query, config.preferences.max_search_results)
}

#[tauri::command]
fn get_note_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name)?;

    let conn = get_db_connection().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT content FROM notes WHERE filename = ?1")
        .map_err(|e| e.to_string())?;

    let content = stmt
        .query_row(params![note_name], |row| Ok(row.get::<_, String>(0)?))
        .map_err(|_| format!("Note not found: {}", note_name))?; // Frontend depends on this exact error message format

    Ok(content)
}

#[tauri::command]
fn get_note_raw_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name)?;

    let conn = get_db_connection().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT content FROM notes WHERE filename = ?1")
        .map_err(|e| e.to_string())?;

    let content = stmt
        .query_row(params![note_name], |row| Ok(row.get::<_, String>(0)?))
        .map_err(|_| format!("Note not found: {}", note_name))?; // Frontend depends on this exact error message format

    Ok(content)
}

#[tauri::command]
fn get_note_html_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name)?;

    let conn = get_db_connection().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT html_render FROM notes WHERE filename = ?1")
        .map_err(|e| e.to_string())?;

    let html_content = stmt
        .query_row(params![note_name], |row| Ok(row.get::<_, String>(0)?))
        .map_err(|_| format!("Note not found: {}", note_name))?; // Frontend depends on this exact error message format

    Ok(html_content)
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

    // 1. Create empty note file atomically (no backup needed for new files)
    safe_write_note(&note_path, "")?;

    // 2. Update database (if this fails, we rebuild from files)
    let modified = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    match get_db_connection() {
        Ok(conn) => {
            // Generate HTML render for empty content
            let html_render = render_note(note_name, "");
            match conn.execute(
                "INSERT OR REPLACE INTO notes (filename, content, html_render, modified) VALUES (?1, ?2, ?3, ?4)",
                params![note_name, "", html_render, modified],
            ) {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!(
                        "Database insert failed for '{}': {}. Rebuilding database...",
                        note_name, e
                    );

                    // Database operation failed - rebuild from markdown files
                    match recreate_database() {
                        Ok(()) => {
                            eprintln!("Database successfully rebuilt from files.");
                            Ok(())
                        }
                        Err(rebuild_error) => {
                            eprintln!("Database rebuild failed: {}. Note was created but may not be searchable.", rebuild_error);
                            // Don't fail the user operation - file was created successfully
                            Ok(())
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Database connection failed for '{}': {}. Rebuilding database...",
                note_name, e
            );

            // Database connection failed - rebuild from markdown files
            match recreate_database() {
                Ok(()) => {
                    eprintln!("Database successfully rebuilt from files.");
                    Ok(())
                }
                Err(rebuild_error) => {
                    eprintln!(
                        "Database rebuild failed: {}. Note was created but may not be searchable.",
                        rebuild_error
                    );
                    // Don't fail the user operation - file was created successfully
                    Ok(())
                }
            }
        }
    }
}

#[tauri::command]
fn save_note(note_name: &str, content: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);

    // Ensure parent directory exists
    if let Some(parent) = note_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // 1. Write file atomically with backup (file operation is primary, never fails the user)
    safe_write_note(&note_path, content)?;

    // 2. Update database (if this fails, we rebuild from files)
    let modified = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    match update_note_in_database(note_name, content, modified) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!(
                "Database update failed for '{}': {}. Rebuilding database...",
                note_name, e
            );

            // Database operation failed - rebuild from markdown files
            match recreate_database() {
                Ok(()) => {
                    eprintln!("Database successfully rebuilt from files.");
                    Ok(())
                }
                Err(rebuild_error) => {
                    eprintln!("Database rebuild failed: {}. Note was saved to file but may not be searchable.", rebuild_error);
                    // Don't fail the user operation - file was saved successfully
                    Ok(())
                }
            }
        }
    }
}

fn update_note_in_database(note_name: &str, content: &str, modified: i64) -> Result<(), String> {
    let conn = get_db_connection()?;

    // Generate HTML render from content
    let html_render = render_note(note_name, content);

    // First try to update existing note
    let updated_rows = conn
        .execute(
            "UPDATE notes SET content = ?2, html_render = ?3, modified = ?4 WHERE filename = ?1",
            params![note_name, content, html_render, modified],
        )
        .map_err(|e| format!("Database error: {}", e))?;

    // If no rows were updated, insert new note
    if updated_rows == 0 {
        conn.execute(
            "INSERT OR REPLACE INTO notes (filename, content, html_render, modified) VALUES (?1, ?2, ?3, ?4)",
            params![note_name, content, html_render, modified],
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

    // If source file doesn't exist, it may have been renamed/deleted externally
    if !old_path.exists() {
        // Check if the target file already exists - this might be an external rename
        if new_path.exists() {
            // File was likely renamed externally - update database to reflect this
            match get_db_connection() {
                Ok(conn) => {
                    let _ = conn.execute(
                        "UPDATE notes SET filename = ?1 WHERE filename = ?2",
                        params![new_name, old_name],
                    );
                }
                Err(_) => {
                    // Database connection failed - trigger rebuild
                    let _ = recreate_database();
                }
            }
            return Ok(()); // Success - rename already happened externally
        } else {
            // File was deleted externally
            return Err(format!("Note '{}' not found", old_name));
        }
    }

    if new_path.exists() {
        return Err(format!("Note '{}' already exists", new_name));
    }

    // 1. Create backup before rename operation
    let backup_path = safe_backup_path(&old_path)?.with_extension("md.bak");

    // Ensure backup directory structure exists
    if let Some(backup_parent) = backup_path.parent() {
        fs::create_dir_all(backup_parent)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    }

    // Copy to backup
    fs::copy(&old_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;

    // 2. Rename the file atomically
    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename note: {}", e))?;

    // 3. Update database (if this fails, we can restore from backup)
    match get_db_connection() {
        Ok(conn) => {
            match conn.execute(
                "UPDATE notes SET filename = ?1 WHERE filename = ?2",
                params![new_name, old_name],
            ) {
                Ok(_) => {
                    // Success - cleanup backup
                    let _ = fs::remove_file(&backup_path);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Database update failed for rename '{}' -> '{}': {}. Rebuilding database...", old_name, new_name, e);

                    // Database operation failed - rebuild from markdown files
                    match recreate_database() {
                        Ok(()) => {
                            eprintln!("Database successfully rebuilt from files.");
                            // Success - cleanup backup
                            let _ = fs::remove_file(&backup_path);
                            Ok(())
                        }
                        Err(rebuild_error) => {
                            eprintln!("Database rebuild failed: {}. Note was renamed but may not be searchable.", rebuild_error);
                            // Don't fail the user operation - file was renamed successfully
                            // Keep backup for potential manual recovery
                            Ok(())
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Database connection failed for rename '{}' -> '{}': {}. Rebuilding database...",
                old_name, new_name, e
            );

            // Database connection failed - rebuild from markdown files
            match recreate_database() {
                Ok(()) => {
                    eprintln!("Database successfully rebuilt from files.");
                    // Success - cleanup backup
                    let _ = fs::remove_file(&backup_path);
                    Ok(())
                }
                Err(rebuild_error) => {
                    eprintln!(
                        "Database rebuild failed: {}. Note was renamed but may not be searchable.",
                        rebuild_error
                    );
                    // Don't fail the user operation - file was renamed successfully
                    // Keep backup for potential manual recovery
                    Ok(())
                }
            }
        }
    }
}

#[tauri::command]
fn delete_note(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);

    // If note doesn't exist on filesystem, just remove from database and return success
    if !note_path.exists() {
        // File was already deleted externally - just clean up database
        match get_db_connection() {
            Ok(conn) => {
                let _ = conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name]);
            }
            Err(_) => {
                // Database connection failed - trigger rebuild
                let _ = recreate_database();
            }
        }
        return Ok(()); // Success - file is already gone
    }

    // 1. Create backup before deletion (for potential recovery)
    let backup_path = safe_backup_path(&note_path)?.with_extension("md.bak.deleted");

    // Ensure backup directory structure exists
    if let Some(backup_parent) = backup_path.parent() {
        fs::create_dir_all(backup_parent)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    }

    // Copy to backup before deletion
    fs::copy(&note_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;

    // 2. Delete the file
    fs::remove_file(&note_path).map_err(|e| format!("Failed to delete note: {}", e))?;

    // 3. Update database (if this fails, we rebuild from remaining files)
    match get_db_connection() {
        Ok(conn) => {
            match conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name]) {
                Ok(_) => {
                    // Success - keep backup for a while in case user wants to restore
                    // Note: We intentionally keep the backup file for potential recovery
                    Ok(())
                }
                Err(e) => {
                    eprintln!(
                        "Database delete failed for '{}': {}. Rebuilding database...",
                        note_name, e
                    );

                    // Database operation failed - rebuild from remaining markdown files
                    match recreate_database() {
                        Ok(()) => {
                            eprintln!("Database successfully rebuilt from files.");
                            Ok(())
                        }
                        Err(rebuild_error) => {
                            eprintln!("Database rebuild failed: {}. Note was deleted but database may be inconsistent.", rebuild_error);
                            // Don't fail the user operation - file was deleted successfully
                            Ok(())
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Database connection failed for delete '{}': {}. Rebuilding database...",
                note_name, e
            );

            // Database connection failed - rebuild from remaining markdown files
            match recreate_database() {
                Ok(()) => {
                    eprintln!("Database successfully rebuilt from files.");
                    Ok(())
                }
                Err(rebuild_error) => {
                    eprintln!("Database rebuild failed: {}. Note was deleted but database may be inconsistent.", rebuild_error);
                    // Don't fail the user operation - file was deleted successfully
                    Ok(())
                }
            }
        }
    }
}

// Tauri command handlers - System operations
#[tauri::command]
async fn refresh_cache(app: AppHandle) -> Result<(), String> {
    // Emit loading start event
    let _ = app.emit("db-loading-start", "Refreshing notes...");
    let _ = app.emit("db-loading-progress", "Loading settings...");

    // Reload config first
    if let Err(e) = reload_config(&APP_CONFIG, Some(app.clone())) {
        let _ = app.emit(
            "db-loading-error",
            format!("Failed to reload config: {}", e),
        );
        return Err(format!("Failed to reload config: {}", e));
    }

    // Check if theme has changed
    let current_theme = {
        let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
        config.interface.md_render_code_theme.clone()
    };

    let theme_changed = {
        let last_theme = LAST_RENDER_THEME.read().unwrap_or_else(|e| e.into_inner());
        match &*last_theme {
            Some(last) => last != &current_theme,
            None => false, // First time, don't force rebuild unless theme actually different from default
        }
    };

    // Update stored theme
    {
        let mut last_theme = LAST_RENDER_THEME.write().unwrap_or_else(|e| e.into_inner());
        *last_theme = Some(current_theme.clone());
    }

    // If theme changed, force complete database rebuild to regenerate all HTML
    if theme_changed {
        let _ = app.emit(
            "db-loading-progress",
            format!(
                "Theme changed to '{}', rebuilding all HTML renders...",
                current_theme
            ),
        );
        eprintln!(
            "Theme changed to '{}', rebuilding all HTML renders...",
            current_theme
        );

        let result = recreate_database_with_progress(
            &app,
            &format!(
                "Theme changed to '{}', rebuilding all HTML renders...",
                current_theme
            ),
        )
        .await;
        if result.is_ok() {
            let _ = app.emit("db-loading-complete", ());
        } else if let Err(ref e) = result {
            let _ = app.emit("db-loading-error", e);
        }
        return result;
    }

    // Otherwise, do normal cache refresh
    let _ = app.emit("db-loading-progress", "Preparing notes database...");
    let mut conn = match get_db_connection() {
        Ok(conn) => conn,
        Err(e) => {
            let _ = app.emit(
                "db-loading-error",
                format!("Database connection error: {}", e),
            );
            return Err(format!("Database connection error: {}", e));
        }
    };

    let _ = app.emit("db-loading-progress", "Setting up notes database...");
    if let Err(e) = init_db(&conn) {
        let _ = app.emit(
            "db-loading-error",
            format!("Database initialization error: {}", e),
        );
        return Err(format!("Database initialization error: {}", e));
    }

    let _ = app.emit("db-loading-progress", "Loading notes...");

    // Use spawn_blocking for CPU-intensive database operations
    let result = tokio::task::spawn_blocking(move || load_all_notes_into_sqlite(&mut conn))
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

    match result {
        Ok(()) => {
            let _ = app.emit("db-loading-complete", ());
            Ok(())
        }
        Err(e) => {
            let _ = app.emit(
                "db-loading-progress",
                "Database sync failed, attempting recovery...",
            );
            eprintln!(
                "Failed to refresh notes cache: {}. Attempting recovery...",
                e
            );

            // Attempt database recovery
            let result = recreate_database_with_progress(
                &app,
                "Database corruption detected. Recreating database tables...",
            )
            .await
            .map_err(|recovery_error| {
                format!(
                    "Cache refresh failed and recovery failed: {}. Original error: {}",
                    recovery_error, e
                )
            });

            if result.is_ok() {
                let _ = app.emit("db-loading-complete", ());
            } else if let Err(ref e) = result {
                let _ = app.emit("db-loading-error", e);
            }
            result
        }
    }
}

#[tauri::command]
fn open_note_in_editor(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);
    if !note_path.exists() {
        return Err(format!("Note not found: {}", note_name)); // Frontend depends on this exact error message format
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
        return Err(format!("Note not found: {}", note_name)); // Frontend depends on this exact error message format
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
            let template = generate_config_template();
            Ok(template)
        }
    }
}

#[tauri::command]
fn config_exists() -> bool {
    get_config_path().exists()
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
    if let Ok(db_path) = get_db_path() {
        if let Some(parent) = db_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
    }

    // Clean up any leftover temp files from previous runs
    let _ = cleanup_temp_files();

    let mut conn = match get_db_connection() {
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
    }

    if !get_config_path().exists() {
        if let Err(e) = conn.execute("DELETE FROM notes", []) {
            eprintln!("Failed to purge database: {}. Continuing anyway.", e);
        }
        return;
    }

    if let Err(e) = load_all_notes_into_sqlite(&mut conn) {
        eprintln!(
            "Failed to load notes into database: {}. Attempting recovery...",
            e
        );

        // Attempt database recovery
        if let Err(recovery_error) = recreate_database() {
            eprintln!(
                "Database recovery failed: {}. Some notes may not be searchable.",
                recovery_error
            );
        }
    }
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
