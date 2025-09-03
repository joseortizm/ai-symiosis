use crate::{
    config::get_config_notes_dir,
    core::{AppError, AppResult},
    database::with_db,
    logging::log,
    search::search_notes_hybrid,
    services::{
        database_service::recreate_database,
        note_service::{
            create_versioned_backup, render_note, safe_write_note, update_note_in_database,
            validate_note_name, BackupType,
        },
    },
    APP_CONFIG, PROGRAMMATIC_OPERATION_IN_PROGRESS,
};
use rusqlite::params;
use std::fs;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Helper function to wrap file operations with programmatic operation flag
fn with_programmatic_flag<T, F>(operation: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T>,
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
    let result = with_db(|conn| {
        let mut stmt = conn.prepare("SELECT filename FROM notes ORDER BY modified DESC")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut results = Vec::new();
        for r in rows {
            if let Ok(filename) = r {
                results.push(filename);
            }
        }

        Ok(results)
    });
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_notes(query: &str) -> Result<Vec<String>, String> {
    let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    let result = search_notes_hybrid(query, config.preferences.max_search_results);
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_note_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name)
        .and_then(|_| {
            with_db(|conn| {
                let mut stmt = conn.prepare("SELECT content FROM notes WHERE filename = ?1")?;
                let content = stmt
                    .query_row(params![note_name], |row| Ok(row.get::<_, String>(0)?))
                    .map_err(|_| {
                        AppError::FileNotFound(format!("Note not found: {}", note_name))
                    })?;
                Ok(content)
            })
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_note_html_content(note_name: &str) -> Result<String, String> {
    validate_note_name(note_name).map_err(|e| e.to_string())?;

    with_db(|conn| {
        let mut stmt =
            conn.prepare("SELECT html_render, is_indexed, content FROM notes WHERE filename = ?1")?;

        let (html_content, is_indexed, content): (String, bool, String) = stmt
            .query_row(params![note_name], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, bool>(1).unwrap_or(false),
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|_| AppError::FileNotFound(format!("Note not found: {}", note_name)))?;

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
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_new_note(note_name: &str) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(note_name)?;

        let note_path = get_config_notes_dir().join(note_name);

        if let Some(parent) = note_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Atomic file creation - this eliminates TOCTOU by using create_new flag
        with_programmatic_flag(|| -> AppResult<()> {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true) // This will fail if file already exists
                .open(&note_path)
            {
                Ok(mut file) => {
                    // File was created successfully, write empty content
                    use std::io::Write;
                    file.write_all(b"")
                        .map_err(|e| AppError::FileWrite(e.to_string()))?;
                    Ok(())
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(
                    AppError::InvalidNoteName(format!("Note '{}' already exists", note_name)),
                ),
                Err(e) => Err(AppError::FileWrite(format!("Failed to create note: {}", e))),
            }
        })?;

        let modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        match with_db(|conn| {
            let html_render = render_note(note_name, "");
            conn.execute(
                "INSERT OR REPLACE INTO notes (filename, content, html_render, modified, is_indexed) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![note_name, "", html_render, modified, true],
            )?;
            Ok(())
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
                        Err(AppError::DatabaseRebuild(format!(
                            "Note created but database rebuild failed: {}",
                            rebuild_error
                        )))
                    }
                }
            }
        }
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_note_with_content_check(
    note_name: &str,
    content: &str,
    original_content: &str,
) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(note_name)?;
        let note_path = get_config_notes_dir().join(note_name);

        // CRITICAL: Validate that destination file hasn't changed since editing began
        // This prevents catastrophic data loss when UI state becomes desynchronized
        let current_content = if note_path.exists() {
            fs::read_to_string(&note_path)?
        } else {
            String::new()
        };

        if current_content != original_content {
            // Content validation failed - create backup of the content that would have been saved
            match create_versioned_backup(&note_path, BackupType::SaveFailure, Some(content)) {
                Ok(backup_path) => {
                    eprintln!(
                        "Created save failure backup due to external modification: {}",
                        backup_path.display()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Failed to create save failure backup for '{}': {}",
                        note_path.display(),
                        e
                    );
                }
            }

            return Err(AppError::InvalidPath(format!(
                "Cannot save '{}': file has been modified since editing began. \
                This safety check prevents accidental data loss.",
                note_name
            )));
        }

        // Content validation passed - proceed with save
        if let Some(parent) = note_path.parent() {
            fs::create_dir_all(parent)?;
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
                        eprintln!("Critical error: {}", rebuild_error);
                        Err(AppError::DatabaseRebuild(format!(
                            "Note saved but database rebuild failed: {}",
                            rebuild_error
                        )))
                    }
                }
            }
        }
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_note(old_name: String, new_name: String) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(&old_name)?;
        validate_note_name(&new_name)?;

        let notes_dir = get_config_notes_dir();
        let old_path = notes_dir.join(&old_name);
        let new_path = notes_dir.join(&new_name);

        // Create backup using unified API - maintains TOCTOU protection through atomic fs::copy
        let backup_result = create_versioned_backup(&old_path, BackupType::Rename, None);

        match backup_result {
            Ok(backup_path) => {
                // Source file exists and backup was created

                // Check if target already exists before rename
                if new_path.exists() {
                    // Clean up backup since we're not proceeding
                    if let Err(e) = fs::remove_file(&backup_path) {
                        log(
                            "BACKUP_CLEANUP",
                            &format!("Failed to remove backup file: {:?}", backup_path),
                            Some(&e.to_string()),
                        );
                    }
                    return Err(AppError::InvalidNoteName(format!(
                        "Note '{}' already exists",
                        new_name
                    )));
                }

                // Create target directory if needed
                if let Some(parent) = new_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Now attempt the atomic rename operation
                let rename_result = with_programmatic_flag(|| {
                    fs::rename(&old_path, &new_path).map_err(AppError::from)
                });

                match rename_result {
                    Ok(()) => {
                        // Rename succeeded - update database and clean up backup
                        match with_db(|conn| {
                            conn.execute(
                                "UPDATE notes SET filename = ?1 WHERE filename = ?2",
                                params![new_name, old_name],
                            )?;
                            Ok(())
                        }) {
                            Ok(_) => {
                                // Database updated successfully - clean up backup
                                if let Err(e) = fs::remove_file(&backup_path) {
                                    log(
                                        "BACKUP_CLEANUP",
                                        &format!("Failed to remove backup file: {:?}", backup_path),
                                        Some(&e.to_string()),
                                    );
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
                            }
                            Err(e) => {
                                eprintln!(
                                    "Database operation failed for rename '{}' -> '{}': {}. Rebuilding database...",
                                    old_name, new_name, e
                                );

                                match recreate_database() {
                                    Ok(()) => {
                                        eprintln!("Database successfully rebuilt from files.");
                                        if let Err(e) = fs::remove_file(&backup_path) {
                                            log(
                                                "BACKUP_CLEANUP",
                                                &format!(
                                                    "Failed to remove backup file: {:?}",
                                                    backup_path
                                                ),
                                                Some(&e.to_string()),
                                            );
                                        }
                                    }
                                    Err(rebuild_error) => {
                                        eprintln!(
                                            "Database rebuild failed: {}. Note was renamed but may not be searchable.",
                                            rebuild_error
                                        );
                                        return Err(AppError::DatabaseRebuild(format!(
                                            "Note renamed but database rebuild failed: {}",
                                            rebuild_error
                                        )));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Rename failed - restore from backup and return error
                        if let Err(restore_err) = fs::rename(&backup_path, &old_path) {
                            eprintln!(
                                "CRITICAL: Failed to restore backup after failed rename: {}",
                                restore_err
                            );
                        }

                        // Determine error type from the rename failure
                        if new_path.exists() {
                            return Err(AppError::InvalidNoteName(format!(
                                "Note '{}' already exists",
                                new_name
                            )));
                        } else {
                            return Err(AppError::FileWrite(format!(
                                "Failed to rename note: {}",
                                e
                            )));
                        }
                    }
                }
            }
            Err(e) => {
                // Source file doesn't exist or is not accessible - handle based on error type
                match &e {
                    AppError::FileNotFound(_) => {
                        // Handle case where file exists only in database
                        if new_path.exists() {
                            match with_db(|conn| {
                                conn.execute(
                                    "UPDATE notes SET filename = ?1 WHERE filename = ?2",
                                    params![new_name, old_name],
                                )?;
                                Ok(())
                            }) {
                                Ok(_) => return Ok(()),
                                Err(_) => {
                                    if let Err(e) = recreate_database() {
                                        log(
                                            "DATABASE_RECOVERY",
                                            "Failed to recreate database during error recovery",
                                            Some(&e.to_string()),
                                        );
                                    }
                                    return Ok(());
                                }
                            }
                        } else {
                            return Err(AppError::FileNotFound(format!(
                                "Note '{}' not found",
                                old_name
                            )));
                        }
                    }
                    _ => {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(note_name: &str) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(note_name)?;
        let note_path = get_config_notes_dir().join(note_name);

        // Create backup using unified API - maintains atomic copy operation for TOCTOU protection
        let copy_result = create_versioned_backup(&note_path, BackupType::Delete, None);

        match copy_result {
            Ok(backup_path) => {
                // File exists and backup was created, now delete the original
                match with_programmatic_flag(|| fs::remove_file(&note_path).map_err(AppError::from))
                {
                    Ok(()) => {
                        // Delete succeeded - log success but keep backup
                        eprintln!(
                            "[{}] File Operation: DELETE | File: {} | Backup: {} | Result: SUCCESS",
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            note_name,
                            backup_path.display()
                        );
                    }
                    Err(e) => {
                        // Delete failed - clean up backup and return error
                        if let Err(e) = fs::remove_file(&backup_path) {
                            log(
                                "BACKUP_CLEANUP",
                                &format!("Failed to remove backup file: {:?}", backup_path),
                                Some(&e.to_string()),
                            );
                        }
                        return Err(AppError::FileWrite(format!("Failed to delete note: {}", e)));
                    }
                }
            }
            Err(_) => {
                // File doesn't exist - handle database-only deletion
                match with_db(|conn| {
                    conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name])?;
                    Ok(())
                }) {
                    Ok(_) => return Ok(()),
                    Err(_) => {
                        if let Err(e) = recreate_database() {
                            log(
                                "DATABASE_RECOVERY",
                                "Failed to recreate database during error recovery",
                                Some(&e.to_string()),
                            );
                        }
                        return Ok(());
                    }
                }
            }
        }

        match with_db(|conn| {
            conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name])?;
            Ok(())
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
                        Err(AppError::DatabaseRebuild(format!(
                            "Note deleted but database rebuild failed: {}",
                            rebuild_error
                        )))
                    }
                }
            }
        }
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_note_in_editor(note_name: &str) -> Result<(), String> {
    validate_note_name(note_name)
        .and_then(|_| {
            let note_path = get_config_notes_dir().join(note_name);
            if !note_path.exists() {
                return Err(AppError::FileNotFound(format!(
                    "Note not found: {}",
                    note_name
                )));
            }

            std::process::Command::new("open")
                .arg(&note_path)
                .status()
                .map_err(AppError::from)?;

            Ok(())
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_note_folder(note_name: &str) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(note_name)?;
        let note_path = get_config_notes_dir().join(note_name);
        if !note_path.exists() {
            return Err(AppError::FileNotFound(format!(
                "Note not found: {}",
                note_name
            )));
        }

        #[cfg(target_os = "macos")]
        std::process::Command::new("open")
            .arg("-R")
            .arg(note_path)
            .status()
            .map_err(AppError::from)?;

        #[cfg(target_os = "windows")]
        {
            let path_str = note_path
                .to_str()
                .ok_or_else(|| AppError::InvalidPath("Invalid path encoding".to_string()))?;
            std::process::Command::new("explorer")
                .arg(format!("/select,\"{}\"", path_str))
                .status()
                .map_err(AppError::from)?;
        }

        #[cfg(target_os = "linux")]
        {
            let folder_path = note_path
                .parent()
                .ok_or_else(|| AppError::InvalidPath("Unable to determine folder".to_string()))?;
            std::process::Command::new("xdg-open")
                .arg(folder_path)
                .status()
                .map_err(AppError::from)?;
        }

        Ok(())
    }();
    result.map_err(|e| e.to_string())
}
