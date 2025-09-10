use crate::{
    core::{AppError, AppResult},
    database::with_db,
    logging::log,
    services::{database_service::handle_database_recovery, note_service::update_note_in_database},
    utilities::{
        file_safety::{create_versioned_backup, safe_write_note, BackupType},
        note_renderer::render_note,
        validation::validate_note_name,
    },
};
use rusqlite::params;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[tauri::command]
pub fn list_all_notes(
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<Vec<String>, String> {
    let result = with_db(&app_state, |conn| {
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
pub fn get_note_content(
    note_name: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<String, String> {
    validate_note_name(note_name)
        .and_then(|_| {
            with_db(&app_state, |conn| {
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
pub fn get_note_html_content(
    note_name: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<String, String> {
    validate_note_name(note_name).map_err(|e| e.to_string())?;

    with_db(&app_state, |conn| {
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
                log(
                    "NOTE_INDEXING",
                    &format!("Failed to update note indexing for '{}'", note_name),
                    Some(&e.to_string()),
                );
            }

            Ok(html_render)
        }
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_new_note(
    note_name: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(note_name)?;

        let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
        let note_path = std::path::PathBuf::from(&config.notes_directory).join(note_name);

        if let Some(parent) = note_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Atomic file creation - this eliminates TOCTOU by using create_new flag
        super::notes::with_programmatic_flag(&app_state, || -> AppResult<()> {
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

        match with_db(&app_state, |conn| {
            let html_render = render_note(note_name, "");
            conn.execute(
                "INSERT OR REPLACE INTO notes (filename, content, html_render, modified, is_indexed) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![note_name, "", html_render, modified, true],
            )?;
            Ok(())
        }) {
            Ok(_) => Ok(()),
            Err(e) => handle_database_recovery(
                &app_state,
                &format!("'{}'", note_name),
                &e,
                "Note created but database rebuild failed",
                "Database rebuild failed. Note was created but may not be searchable.",
            ),
        }
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_note_with_content_check(
    note_name: &str,
    content: &str,
    original_content: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(note_name)?;
        let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
        let note_path = std::path::PathBuf::from(&config.notes_directory).join(note_name);

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
                    log(
                        "FILE_BACKUP",
                        "Created save failure backup due to external modification",
                        Some(&backup_path.display().to_string()),
                    );
                }
                Err(e) => {
                    log(
                        "FILE_BACKUP",
                        &format!(
                            "Failed to create save failure backup for '{}'",
                            note_path.display()
                        ),
                        Some(&e.to_string()),
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

        super::notes::with_programmatic_flag(&app_state, || safe_write_note(&note_path, content))?;

        let modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        match update_note_in_database(&app_state, note_name, content, modified) {
            Ok(()) => Ok(()),
            Err(e) => handle_database_recovery(
                &app_state,
                &format!("update '{}'", note_name),
                &e,
                "Note saved but database rebuild failed",
                "Critical error: Database rebuild failed",
            ),
        }
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_note(
    old_name: String,
    new_name: String,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(&old_name)?;
        validate_note_name(&new_name)?;

        let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
        let notes_dir = std::path::PathBuf::from(&config.notes_directory);
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
                let rename_result = super::notes::with_programmatic_flag(&app_state, || {
                    fs::rename(&old_path, &new_path).map_err(AppError::from)
                });

                match rename_result {
                    Ok(()) => {
                        // Rename succeeded - update database and clean up backup
                        match with_db(&app_state, |conn| {
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
                                log(
                                    "FILE_OPERATION",
                                    &format!("RENAME: {} -> {} | SUCCESS", old_name, new_name),
                                    None,
                                );
                            }
                            Err(e) => {
                                if let Err(_) = handle_database_recovery(
                                    &app_state,
                                    &format!("rename '{}' -> '{}'", old_name, new_name),
                                    &e,
                                    "Note renamed but database rebuild failed",
                                    "Database rebuild failed. Note was renamed but may not be searchable.",
                                ) {
                                    return Err(AppError::DatabaseRebuild(format!(
                                        "Note renamed but database rebuild failed: {}",
                                        e
                                    )));
                                }
                                if let Err(e) = fs::remove_file(&backup_path) {
                                    crate::logging::log(
                                        "BACKUP_CLEANUP",
                                        &format!("Failed to remove backup file: {:?}", backup_path),
                                        Some(&e.to_string()),
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Rename failed - restore from backup and return error
                        if let Err(restore_err) = fs::rename(&backup_path, &old_path) {
                            log(
                                "FILE_OPERATION",
                                "CRITICAL: Failed to restore backup after failed rename",
                                Some(&restore_err.to_string()),
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
                            match with_db(&app_state, |conn| {
                                conn.execute(
                                    "UPDATE notes SET filename = ?1 WHERE filename = ?2",
                                    params![new_name, old_name],
                                )?;
                                Ok(())
                            }) {
                                Ok(_) => return Ok(()),
                                Err(e) => {
                                    let _ = handle_database_recovery(
                                        &app_state,
                                        "database-only rename recovery",
                                        &e,
                                        "Database recovery completed",
                                        "Failed to recreate database during error recovery",
                                    );
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
pub fn delete_note(
    note_name: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(note_name)?;
        let config = app_state.config.read().unwrap_or_else(|e| {
            log(
                "DELETE_NOTE",
                "Config lock was poisoned, recovering",
                Some(&format!("note: {}", note_name)),
            );
            e.into_inner()
        });
        let note_path = std::path::PathBuf::from(&config.notes_directory).join(note_name);

        log(
            "DELETE_NOTE",
            "Critical filesystem operation initiated",
            Some(&format!(
                "note: {}, directory: {}",
                note_name, config.notes_directory
            )),
        );

        // Create backup using unified API - maintains atomic copy operation for TOCTOU protection
        let copy_result = create_versioned_backup(&note_path, BackupType::Delete, None);

        match copy_result {
            Ok(backup_path) => {
                // File exists and backup was created, now delete the original
                match super::notes::with_programmatic_flag(&app_state, || {
                    fs::remove_file(&note_path).map_err(AppError::from)
                }) {
                    Ok(()) => {
                        // Delete succeeded - log success but keep backup
                        log(
                            "FILE_OPERATION",
                            &format!(
                                "DELETE: {} | Backup: {} | SUCCESS",
                                note_name,
                                backup_path.display()
                            ),
                            None,
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
                match with_db(&app_state, |conn| {
                    conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name])?;
                    Ok(())
                }) {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        let _ = handle_database_recovery(
                            &app_state,
                            "database-only delete recovery",
                            &e,
                            "Database recovery completed",
                            "Failed to recreate database during error recovery",
                        );
                        return Ok(());
                    }
                }
            }
        }

        match with_db(&app_state, |conn| {
            conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name])?;
            Ok(())
        }) {
            Ok(_) => Ok(()),
            Err(e) => handle_database_recovery(
                &app_state,
                &format!("delete '{}'", note_name),
                &e,
                "Note deleted but database rebuild failed",
                "Database rebuild failed. Note was deleted but database may be inconsistent.",
            ),
        }
    }();
    result.map_err(|e| e.to_string())
}
