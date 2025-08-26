use crate::{
    database::with_db,
    get_config_notes_dir, recreate_database, render_note, safe_backup_path, safe_write_note,
    search::search_notes_hybrid,
    update_note_in_database, validate_note_name, APP_CONFIG, PROGRAMMATIC_OPERATION_IN_PROGRESS,
};
use rusqlite::params;
use std::fs;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Helper function to wrap file operations with programmatic operation flag
fn with_programmatic_flag<T, F>(operation: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    // Set flag immediately
    PROGRAMMATIC_OPERATION_IN_PROGRESS.store(true, Ordering::SeqCst);

    // Execute operation (this is still synchronous and fast)
    let result = operation();

    // Spawn background thread to clear flag after delay - NON-BLOCKING
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(5)); // Long enough for watcher to process
        PROGRAMMATIC_OPERATION_IN_PROGRESS.store(false, Ordering::SeqCst);
    });

    result
}

#[tauri::command]
pub fn list_all_notes() -> Result<Vec<String>, String> {
    with_db(|conn| {
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
    })
}

#[tauri::command]
pub fn search_notes(query: &str) -> Result<Vec<String>, String> {
    let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    search_notes_hybrid(query, config.preferences.max_search_results)
}

#[tauri::command]
pub fn get_note_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name)?;

    with_db(|conn| {
        let mut stmt = conn
            .prepare("SELECT content FROM notes WHERE filename = ?1")
            .map_err(|e| e.to_string())?;

        let content = stmt
            .query_row(params![note_name], |row| Ok(row.get::<_, String>(0)?))
            .map_err(|_| format!("Note not found: {}", note_name))?;

        Ok(content)
    })
}

#[tauri::command]
pub fn get_note_raw_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name)?;

    with_db(|conn| {
        let mut stmt = conn
            .prepare("SELECT content FROM notes WHERE filename = ?1")
            .map_err(|e| e.to_string())?;

        let content = stmt
            .query_row(params![note_name], |row| Ok(row.get::<_, String>(0)?))
            .map_err(|_| format!("Note not found: {}", note_name))?;

        Ok(content)
    })
}

#[tauri::command]
pub fn get_note_html_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name)?;

    with_db(|conn| {
        let mut stmt = conn
            .prepare("SELECT html_render, is_indexed, content FROM notes WHERE filename = ?1")
            .map_err(|e| e.to_string())?;

        let (html_content, is_indexed, content): (String, bool, String) = stmt
            .query_row(params![note_name], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, bool>(1).unwrap_or(false),
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|_| format!("Note not found: {}", note_name))?;

        if is_indexed {
            Ok(html_content)
        } else {
            let html_render = render_note(note_name, &content);

            if let Err(e) = conn.execute(
                "UPDATE notes SET html_render = ?2, is_indexed = ?3 WHERE filename = ?1",
                params![note_name, html_render, true],
            ) {
                eprintln!("Failed to update note indexing for '{}': {}", note_name, e);
            }

            Ok(html_render)
        }
    })
}

#[tauri::command]
pub fn create_new_note(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)?;

    let note_path = get_config_notes_dir().join(note_name);

    if note_path.exists() {
        return Err(format!("Note '{}' already exists", note_name));
    }

    if let Some(parent) = note_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    with_programmatic_flag(|| safe_write_note(&note_path, ""))?;

    let modified = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    match with_db(|conn| {
        let html_render = render_note(note_name, "");
        conn.execute(
            "INSERT OR REPLACE INTO notes (filename, content, html_render, modified, is_indexed) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![note_name, "", html_render, modified, true],
        ).map_err(|e| format!("Database insert failed: {}", e))
    }) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "Database operation failed for '{}': {}. Rebuilding database...",
                note_name, e
            );

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
                    Ok(())
                }
            }
        }
    }
}

#[tauri::command]
pub fn save_note(note_name: &str, content: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);

    if let Some(parent) = note_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    with_programmatic_flag(|| safe_write_note(&note_path, content))?;

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

            match recreate_database() {
                Ok(()) => {
                    eprintln!("Database successfully rebuilt from files.");
                    Ok(())
                }
                Err(rebuild_error) => {
                    eprintln!("Database rebuild failed: {}. Note was saved to file but may not be searchable.", rebuild_error);
                    Ok(())
                }
            }
        }
    }
}

#[tauri::command]
pub fn rename_note(old_name: String, new_name: String) -> Result<(), String> {
    validate_note_name(&old_name)?;
    validate_note_name(&new_name)?;

    let notes_dir = get_config_notes_dir();
    let old_path = notes_dir.join(&old_name);
    let new_path = notes_dir.join(&new_name);

    // Pre-flight checks
    if !old_path.exists() {
        if new_path.exists() {
            match with_db(|conn| {
                conn.execute(
                    "UPDATE notes SET filename = ?1 WHERE filename = ?2",
                    params![new_name, old_name],
                )
                .map_err(|e| e.to_string())
            }) {
                Ok(_) => {}
                Err(_) => {
                    let _ = recreate_database();
                }
            }
            return Ok(());
        } else {
            return Err(format!("Note '{}' not found", old_name));
        }
    }

    if new_path.exists() {
        return Err(format!("Note '{}' already exists", new_name));
    }

    // Additional pre-flight checks
    if let Ok(metadata) = old_path.metadata() {
        if !metadata.permissions().readonly() == false {
            return Err(format!("Source file '{}' is not readable", old_name));
        }
    } else {
        return Err(format!("Cannot access source file '{}'", old_name));
    }

    let backup_path = safe_backup_path(&old_path)?.with_extension("md.bak");

    if let Some(backup_parent) = backup_path.parent() {
        fs::create_dir_all(backup_parent)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    }

    fs::copy(&old_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;

    with_programmatic_flag(|| {
        fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename note: {}", e))
    })?;

    // Post-operation verification
    if !new_path.exists() {
        return Err(format!(
            "Rename operation failed - destination file '{}' not found",
            new_name
        ));
    }

    if old_path.exists() {
        return Err(format!(
            "Rename operation failed - source file '{}' still exists",
            old_name
        ));
    }

    // Log successful rename operation
    eprintln!(
        "[{}] File Operation: RENAME | From: {} | To: {} | Result: SUCCESS",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        old_name,
        new_name
    );

    match with_db(|conn| {
        conn.execute(
            "UPDATE notes SET filename = ?1 WHERE filename = ?2",
            params![new_name, old_name],
        )
        .map(|_| {
            let _ = fs::remove_file(&backup_path);
        })
        .map_err(|e| format!("Database update failed: {}", e))
    }) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "Database operation failed for rename '{}' -> '{}': {}. Rebuilding database...",
                old_name, new_name, e
            );

            match recreate_database() {
                Ok(()) => {
                    eprintln!("Database successfully rebuilt from files.");
                    let _ = fs::remove_file(&backup_path);
                    Ok(())
                }
                Err(rebuild_error) => {
                    eprintln!(
                        "Database rebuild failed: {}. Note was renamed but may not be searchable.",
                        rebuild_error
                    );
                    Ok(())
                }
            }
        }
    }
}

#[tauri::command]
pub fn delete_note(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);

    if !note_path.exists() {
        match with_db(|conn| {
            conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name])
                .map_err(|e| e.to_string())
        }) {
            Ok(_) => {}
            Err(_) => {
                let _ = recreate_database();
            }
        }
        return Ok(());
    }

    let backup_path = safe_backup_path(&note_path)?.with_extension("md.bak.deleted");

    if let Some(backup_parent) = backup_path.parent() {
        fs::create_dir_all(backup_parent)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    }

    fs::copy(&note_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;

    with_programmatic_flag(|| {
        fs::remove_file(&note_path).map_err(|e| format!("Failed to delete note: {}", e))
    })?;

    // Post-operation verification
    if note_path.exists() {
        return Err(format!(
            "Delete operation failed - file '{}' still exists",
            note_name
        ));
    }

    if !backup_path.exists() {
        return Err(format!(
            "Delete operation completed but backup was not created for '{}'",
            note_name
        ));
    }

    // Log successful delete operation
    eprintln!(
        "[{}] File Operation: DELETE | File: {} | Backup: {} | Result: SUCCESS",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        note_name,
        backup_path.display()
    );

    match with_db(|conn| {
        conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name])
            .map_err(|e| format!("Database delete failed: {}", e))
    }) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "Database operation failed for delete '{}': {}. Rebuilding database...",
                note_name, e
            );

            match recreate_database() {
                Ok(()) => {
                    eprintln!("Database successfully rebuilt from files.");
                    Ok(())
                }
                Err(rebuild_error) => {
                    eprintln!("Database rebuild failed: {}. Note was deleted but database may be inconsistent.", rebuild_error);
                    Ok(())
                }
            }
        }
    }
}

#[tauri::command]
pub fn open_note_in_editor(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)?;
    let note_path = get_config_notes_dir().join(note_name);
    if !note_path.exists() {
        return Err(format!("Note not found: {}", note_name));
    }

    std::process::Command::new("open")
        .arg(&note_path)
        .status()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn open_note_folder(note_name: &str) -> Result<(), String> {
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
            .arg(format!("/select,\"{}\"", path_str))
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
