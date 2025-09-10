use crate::{
    core::{AppError, AppResult},
    services::note_service::update_note_in_database,
    utilities::{
        file_safety::safe_write_note,
        strings::{
            format_timestamp_for_humans, parse_backup_filename, parse_deleted_backup_filename,
        },
        validation::validate_note_name,
    },
};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(serde::Serialize)]
pub struct NoteVersion {
    pub filename: String,
    pub backup_type: String,
    pub timestamp: u64,
    pub size: u64,
    pub formatted_time: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DeletedFile {
    pub filename: String,
    pub backup_filename: String,
    pub deleted_at: String,
    pub timestamp: u64,
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

                if let Some((backup_type, timestamp)) = parse_backup_filename(&filename, &base_name)
                {
                    if let Ok(metadata) = entry.metadata() {
                        let size = metadata.len();
                        let formatted_time = format_timestamp_for_humans(timestamp);

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
        super::notes::with_programmatic_flag(&app_state, || {
            safe_write_note(&note_path, &version_content)
        })?;

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

                if let Some((original_filename, timestamp)) =
                    parse_deleted_backup_filename(&filename)
                {
                    let formatted_time = format_timestamp_for_humans(timestamp);

                    deleted_files.push(DeletedFile {
                        filename: original_filename,
                        backup_filename: filename,
                        deleted_at: formatted_time,
                        timestamp,
                    });
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
            crate::logging::log(
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

        crate::logging::log(
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
        super::notes::with_programmatic_flag(&app_state, || {
            safe_write_note(&note_path, &backup_content)
        })?;

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
