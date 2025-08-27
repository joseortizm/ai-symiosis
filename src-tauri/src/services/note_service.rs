use crate::{
    config::get_config_notes_dir,
    core::{AppError, AppResult},
    database::{get_backup_dir_for_notes_path, get_temp_dir, with_db},
};
use html_escape;
use pulldown_cmark::{html, Options, Parser};
use rusqlite::params;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn validate_note_name(note_name: &str) -> AppResult<()> {
    // Check for empty name
    if note_name.trim().is_empty() {
        return Err(AppError::InvalidNoteName(
            "Note name cannot be empty".to_string(),
        ));
    }
    // Prevent path traversal attacks
    if std::path::Path::new(note_name)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(AppError::PathTraversal);
    }

    if note_name.contains('\\') {
        return Err(AppError::InvalidNoteName("Invalid note name".to_string()));
    }

    if std::path::Path::new(note_name).is_absolute() {
        return Err(AppError::InvalidNoteName(
            "Absolute paths not allowed".to_string(),
        ));
    }
    // Prevent hidden files and system files
    if note_name.starts_with('.') {
        return Err(AppError::InvalidNoteName(
            "Note name cannot start with a dot".to_string(),
        ));
    }
    // Prevent excessively long names
    if note_name.len() > 255 {
        return Err(AppError::InvalidNoteName("Note name too long".to_string()));
    }
    Ok(())
}

pub fn render_note(filename: &str, content: &str) -> String {
    if filename.ends_with(".md") || filename.ends_with(".markdown") {
        // Configure pulldown-cmark options
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);

        // Parse markdown and convert to HTML
        let parser = Parser::new_ext(content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    } else {
        format!("<pre>{}</pre>", html_escape::encode_text(content))
    }
}

// How many backup versions we keep
const MAX_BACKUPS: usize = 20;

pub fn safe_write_note(note_path: &PathBuf, content: &str) -> AppResult<()> {
    // 1. Create backup if file exists
    if note_path.exists() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut backup_path = safe_backup_path(note_path)?;
        let backup_filename = format!(
            "{}.{}.bak",
            backup_path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| AppError::InvalidPath("Invalid filename".to_string()))?,
            timestamp
        );
        backup_path.set_file_name(backup_filename);

        if let Some(backup_parent) = backup_path.parent() {
            fs::create_dir_all(backup_parent)?;
        }

        fs::copy(note_path, &backup_path)?;

        prune_old_backups(&backup_path, MAX_BACKUPS)?;
    }

    // 2. Create temp file in app data directory
    let temp_dir = get_temp_dir()?;
    fs::create_dir_all(&temp_dir)?;

    // Generate unique temp filename using timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_path = temp_dir.join(format!("write_temp_{}.md", timestamp));

    // 3. Write content to temp file
    fs::write(&temp_path, content)?;

    // 4. Atomic rename to final location
    fs::rename(&temp_path, note_path)?;

    // Log successful operation
    eprintln!(
        "[{}] File Operation: WRITE | File: {} | Size: {} bytes | Result: SUCCESS",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        note_path.display(),
        content.len()
    );

    // 5. Verify content was written correctly
    let written_content = fs::read_to_string(note_path)
        .map_err(|e| format!("Failed to verify written content: {}", e))?;

    if written_content != content {
        let error_msg = format!(
            "Content verification failed for '{}': expected {} bytes, found {} bytes",
            note_path.display(),
            content.len(),
            written_content.len()
        );
        eprintln!(
            "[{}] {}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            error_msg
        );
        return Err(AppError::FileWrite(error_msg));
    }

    Ok(())
}

fn prune_old_backups(latest_backup: &PathBuf, max_backups: usize) -> AppResult<()> {
    let parent = latest_backup.parent().ok_or_else(|| {
        AppError::InvalidPath("Failed to get backup parent directory".to_string())
    })?;

    let stem = latest_backup
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidPath("Invalid backup stem".to_string()))?;

    // Strip trailing ".<timestamp>" from stem
    let base_name = stem.rsplitn(2, '.').last().unwrap_or(stem);

    let mut backups: Vec<_> = fs::read_dir(parent)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|f| f.starts_with(base_name) && f.ends_with(".bak"))
                .unwrap_or(false)
        })
        .collect();

    backups.sort_by_key(|e| e.file_name());

    if backups.len() > max_backups {
        for old in &backups[..backups.len() - max_backups] {
            let _ = fs::remove_file(old.path());
        }
    }

    Ok(())
}

pub fn safe_backup_path(note_path: &PathBuf) -> AppResult<PathBuf> {
    let notes_dir = get_config_notes_dir();
    let backup_dir = get_backup_dir_for_notes_path(&notes_dir)?;

    // Get relative path from notes directory to preserve folder structure
    let relative_path = note_path.strip_prefix(&notes_dir).map_err(|_| {
        AppError::InvalidPath(format!(
            "Note path '{}' is not within configured notes directory '{}'",
            note_path.display(),
            notes_dir.display()
        ))
    })?;

    Ok(backup_dir.join(relative_path))
}

pub fn cleanup_temp_files() -> AppResult<()> {
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

pub fn update_note_in_database(
    note_name: &str,
    content: &str,
    modified: i64,
) -> Result<(), String> {
    with_db(|conn| {
        // Generate HTML render from content
        let html_render = render_note(note_name, content);

        // First try to update existing note
        let updated_rows = conn
            .execute(
                "UPDATE notes SET content = ?2, html_render = ?3, modified = ?4, is_indexed = ?5 WHERE filename = ?1",
                params![note_name, content, html_render, modified, true],
            )
            .map_err(|e| format!("Database error: {}", e))?;

        // If no rows were updated, insert new note
        if updated_rows == 0 {
            conn.execute(
                "INSERT OR REPLACE INTO notes (filename, content, html_render, modified, is_indexed) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![note_name, content, html_render, modified, true],
            )
            .map_err(|e| format!("Database error: {}", e))?;
        }

        // Verify database was updated correctly
        let db_content = conn
            .query_row(
                "SELECT content FROM notes WHERE filename = ?1",
                params![note_name],
                |row| row.get::<_, String>(0),
            )
            .map_err(|e| format!("Failed to verify database update: {}", e))?;

        if db_content != content {
            let error_msg = format!(
                "Database update verification failed for '{}': expected {} bytes, found {} bytes",
                note_name,
                content.len(),
                db_content.len()
            );
            eprintln!(
                "[{}] {}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                error_msg
            );
            return Err(AppError::DatabaseQuery(error_msg));
        }

        // Log successful database operation
        eprintln!(
            "[{}] Database Operation: UPDATE/INSERT | File: {} | Size: {} bytes | Result: SUCCESS",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            note_name,
            content.len()
        );

        Ok(())
    }).map_err(|e| e.to_string())
}
