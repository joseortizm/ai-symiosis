use crate::{
    core::{AppError, AppResult},
    database::with_db,
    logging::log,
    search::search_notes_hybrid,
    services::{
        database_service::recreate_database,
        note_service::{
            create_versioned_backup, safe_write_note, update_note_in_database, BackupType,
        },
    },
    utilities::{note_renderer::render_note, validation::validate_note_name},
};
use rusqlite::params;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Helper function to wrap file operations with programmatic operation flag
fn with_programmatic_flag<T, F>(
    app_state: &crate::core::state::AppState,
    operation: F,
) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T>,
{
    // Set flag immediately
    app_state
        .programmatic_operation_in_progress()
        .store(true, std::sync::atomic::Ordering::Relaxed);

    // Execute operation (this is still synchronous and fast)
    let result = operation();

    // Spawn background thread to clear flag after delay - NON-BLOCKING
    let prog_flag = Arc::clone(&app_state.programmatic_operation_in_progress);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5)); // Long enough for watcher to process
        prog_flag.store(false, std::sync::atomic::Ordering::Relaxed);
    });

    result
}

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
pub fn search_notes(
    query: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<Vec<String>, String> {
    let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
    search_notes_hybrid(&app_state, query, config.preferences.max_search_results)
        .map_err(|e| e.to_string())
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
                eprintln!("Failed to update note indexing for '{}': {}", note_name, e);
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
        with_programmatic_flag(&app_state, || -> AppResult<()> {
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
            Err(e) => {
                eprintln!(
                    "Database operation failed for '{}': {}. Rebuilding database...",
                    note_name, e
                );

                match recreate_database(&app_state) {
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

        with_programmatic_flag(&app_state, || safe_write_note(&note_path, content))?;

        let modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        match update_note_in_database(&app_state, note_name, content, modified) {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!(
                    "Database update failed for '{}': {}. Rebuilding database...",
                    note_name, e
                );

                match recreate_database(&app_state) {
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
                let rename_result = with_programmatic_flag(&app_state, || {
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

                                match recreate_database(&app_state) {
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
                            match with_db(&app_state, |conn| {
                                conn.execute(
                                    "UPDATE notes SET filename = ?1 WHERE filename = ?2",
                                    params![new_name, old_name],
                                )?;
                                Ok(())
                            }) {
                                Ok(_) => return Ok(()),
                                Err(_) => {
                                    if let Err(e) = recreate_database(&app_state) {
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
                match with_programmatic_flag(&app_state, || {
                    fs::remove_file(&note_path).map_err(AppError::from)
                }) {
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
                match with_db(&app_state, |conn| {
                    conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name])?;
                    Ok(())
                }) {
                    Ok(_) => return Ok(()),
                    Err(_) => {
                        if let Err(e) = recreate_database(&app_state) {
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

        match with_db(&app_state, |conn| {
            conn.execute("DELETE FROM notes WHERE filename = ?1", params![note_name])?;
            Ok(())
        }) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!(
                    "Database operation failed for delete '{}': {}. Rebuilding database...",
                    note_name, e
                );

                match recreate_database(&app_state) {
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
pub fn open_note_in_editor(
    note_name: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<(), String> {
    validate_note_name(note_name)
        .and_then(|_| {
            let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
            let note_path = std::path::PathBuf::from(&config.notes_directory).join(note_name);
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
pub fn open_note_folder(
    note_name: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(note_name)?;
        let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
        let note_path = std::path::PathBuf::from(&config.notes_directory).join(note_name);
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

#[derive(serde::Serialize)]
pub struct NoteVersion {
    pub filename: String,
    pub backup_type: String,
    pub timestamp: u64,
    pub size: u64,
    pub formatted_time: String,
}

#[tauri::command]
pub fn get_note_versions(
    note_name: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<Vec<NoteVersion>, String> {
    let result = || -> AppResult<Vec<NoteVersion>> {
        validate_note_name(note_name)?;

        let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
        let notes_dir = std::path::PathBuf::from(&config.notes_directory);
        let backup_dir = crate::database::get_backup_dir_for_notes_path(&notes_dir)?;
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let base_name = if let Some(stem) = std::path::Path::new(note_name).file_stem() {
            stem.to_string_lossy()
        } else {
            std::borrow::Cow::from(note_name)
        };

        let mut versions = Vec::new();

        if let Ok(entries) = fs::read_dir(&backup_dir) {
            for entry in entries.flatten() {
                let filename = entry.file_name().to_string_lossy().to_string();

                // Parse backup filename format: {base_name}.{suffix}.{timestamp}.md
                let parts: Vec<&str> = filename.splitn(4, '.').collect();
                if parts.len() == 4 && parts[0] == base_name && parts[3] == "md" {
                    let backup_type = parts[1].to_string();
                    if let Ok(timestamp) = parts[2].parse::<u64>() {
                        if let Ok(metadata) = entry.metadata() {
                            let size = metadata.len();

                            // Format timestamp for display
                            let formatted_time = format_timestamp(timestamp);

                            versions.push(NoteVersion {
                                filename: filename.clone(),
                                backup_type,
                                timestamp,
                                size,
                                formatted_time,
                            });
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(versions)
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_version_content(
    version_filename: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<String, String> {
    let result = || -> AppResult<String> {
        let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
        let notes_dir = std::path::PathBuf::from(&config.notes_directory);
        let backup_dir = crate::database::get_backup_dir_for_notes_path(&notes_dir)?;
        let version_path = backup_dir.join(version_filename);

        if !version_path.exists() {
            return Err(AppError::FileNotFound(format!(
                "Version file not found: {}",
                version_filename
            )));
        }

        let content = fs::read_to_string(&version_path)?;
        Ok(content)
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn recover_note_version(
    note_name: &str,
    version_filename: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(note_name)?;

        let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
        let notes_dir = std::path::PathBuf::from(&config.notes_directory);
        let note_path = notes_dir.join(note_name);
        let backup_dir = crate::database::get_backup_dir_for_notes_path(&notes_dir)?;
        let version_path = backup_dir.join(version_filename);

        if !version_path.exists() {
            return Err(AppError::FileNotFound(format!(
                "Version file not found: {}",
                version_filename
            )));
        }

        // Read the version content
        let version_content = fs::read_to_string(&version_path)?;

        // Use the same programmatic flag and safe write as normal saves
        with_programmatic_flag(&app_state, || safe_write_note(&note_path, &version_content))?;

        let modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Update database with recovered content
        update_note_in_database(&app_state, note_name, &version_content, modified)?;

        Ok(())
    }();
    result.map_err(|e| e.to_string())
}

fn format_timestamp(timestamp: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(timestamp);

    match diff {
        0..=59 => "Just now".to_string(),
        60..=3599 => format!("{}m ago", diff / 60),
        3600..=86399 => format!("{}h ago", diff / 3600),
        86400..=2591999 => format!("{}d ago", diff / 86400),
        _ => format!("{}w ago", diff / 604800),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DeletedFile {
    pub filename: String,
    pub backup_filename: String,
    pub deleted_at: String,
    pub timestamp: u64,
}

#[tauri::command]
pub fn get_deleted_files(
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<Vec<DeletedFile>, String> {
    let result = || -> AppResult<Vec<DeletedFile>> {
        let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
        let notes_dir = std::path::PathBuf::from(&config.notes_directory);
        let backup_dir = crate::database::get_backup_dir_for_notes_path(&notes_dir)?;
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut deleted_files = Vec::new();

        if let Ok(entries) = fs::read_dir(&backup_dir) {
            for entry in entries.flatten() {
                let filename = entry.file_name().to_string_lossy().to_string();

                // Parse backup filename format: {base_name}.delete_backup.{timestamp}.md
                let parts: Vec<&str> = filename.splitn(4, '.').collect();
                if parts.len() == 4 && parts[1] == "delete_backup" && parts[3] == "md" {
                    if let Ok(timestamp) = parts[2].parse::<u64>() {
                        let original_filename = format!("{}.md", parts[0]);
                        let formatted_time = format_timestamp(timestamp);

                        deleted_files.push(DeletedFile {
                            filename: original_filename,
                            backup_filename: filename,
                            deleted_at: formatted_time,
                            timestamp,
                        });
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        deleted_files.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(deleted_files)
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn recover_deleted_file(
    original_filename: &str,
    backup_filename: &str,
    app_state: tauri::State<crate::core::state::AppState>,
) -> Result<(), String> {
    let result = || -> AppResult<()> {
        validate_note_name(original_filename)?;

        let config = app_state.config.read().unwrap_or_else(|e| {
            log(
                "RECOVER_FILE",
                "Config lock was poisoned, recovering",
                Some(&format!(
                    "original: {}, backup: {}",
                    original_filename, backup_filename
                )),
            );
            e.into_inner()
        });
        let notes_dir = std::path::PathBuf::from(&config.notes_directory);

        log(
            "RECOVER_FILE",
            "Critical filesystem recovery operation initiated",
            Some(&format!(
                "original: {}, backup: {}, directory: {}",
                original_filename, backup_filename, config.notes_directory
            )),
        );
        let note_path = notes_dir.join(original_filename);
        let backup_dir = crate::database::get_backup_dir_for_notes_path(&notes_dir)?;
        let backup_path = backup_dir.join(backup_filename);

        if !backup_path.exists() {
            return Err(AppError::FileNotFound(format!(
                "Deleted file backup not found: {}",
                backup_filename
            )));
        }

        // Check if target file already exists
        if note_path.exists() {
            return Err(AppError::FileWrite(format!(
                "Cannot recover '{}': file already exists",
                original_filename
            )));
        }

        // Read the backup content
        let backup_content = fs::read_to_string(&backup_path)?;

        // Write to the original location
        with_programmatic_flag(&app_state, || safe_write_note(&note_path, &backup_content))?;

        let modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Update database with recovered content
        update_note_in_database(&app_state, original_filename, &backup_content, modified)?;

        // Remove the backup file after successful recovery
        fs::remove_file(&backup_path)?;

        Ok(())
    }();
    result.map_err(|e| e.to_string())
}
