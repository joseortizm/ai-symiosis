use crate::{
    config::get_config_notes_dir,
    core::{AppError, AppResult},
    database::{with_db, with_db_mut},
    logging::log,
    services::note_service,
};
use rusqlite::{params, Connection};
use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::{LazyLock, Mutex},
    time::UNIX_EPOCH,
};
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

// Global database lock to prevent concurrent database operations
static DB_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

// Critical database rebuild lock - prevents ALL database operations during rebuild
static DATABASE_REBUILD_LOCK: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

pub fn is_database_rebuilding() -> bool {
    DATABASE_REBUILD_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

fn set_database_rebuilding(rebuilding: bool) {
    *DATABASE_REBUILD_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = rebuilding;
}

// Number of most recent notes to get immediate HTML rendering during startup
// Remaining notes get metadata-only and are processed on demand
const IMMEDIATE_RENDER_COUNT: usize = 2000;

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS notes USING fts5(filename, content, html_render, modified UNINDEXED, is_indexed UNINDEXED);")?;

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

pub fn load_all_notes_into_sqlite(conn: &mut Connection) -> rusqlite::Result<()> {
    load_all_notes_into_sqlite_with_progress(conn, None)
}

pub fn load_all_notes_into_sqlite_with_progress(
    conn: &mut Connection,
    app_handle: Option<&AppHandle>,
) -> rusqlite::Result<()> {
    let _lock = DB_LOCK.lock().map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            Some(format!("Operation lock poisoned: {}", e)),
        )
    })?;

    let notes_dir = get_config_notes_dir();

    if !notes_dir.exists() {
        if let Err(e) = fs::create_dir_all(&notes_dir) {
            eprintln!("Failed to create notes directory: {}", e);
            return Ok(());
        }
    }

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

    filesystem_files.sort_by(|a, b| b.2.cmp(&a.2));

    let mut database_files = HashMap::new();
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

    let filesystem_filenames: HashSet<_> =
        filesystem_files.iter().map(|(name, _, _)| name).collect();
    for filename in database_files.keys() {
        if !filesystem_filenames.contains(filename) {
            tx.execute("DELETE FROM notes WHERE filename = ?1", params![filename])?;
        }
    }

    let total_files = filesystem_files.len();

    for (index, (filename, path, fs_modified)) in filesystem_files.iter().enumerate() {
        if let Some(app) = app_handle {
            if index == 0 || (index + 1) % 10 == 0 || index == total_files - 1 {
                let progress_msg = format!("Loading {} of {} notes...", index + 1, total_files);
                if let Err(e) = app.emit("db-loading-progress", progress_msg) {
                    log(
                        "UI_UPDATE",
                        "Failed to emit db-loading-progress event",
                        Some(&e.to_string()),
                    );
                }
            }
        }

        let (db_modified, is_indexed) = database_files.get(filename).copied().unwrap_or((0, false));

        if *fs_modified != db_modified {
            let content = fs::read_to_string(path).unwrap_or_default();

            if index < IMMEDIATE_RENDER_COUNT {
                let html_render = note_service::render_note(filename, &content);
                tx.execute(
                    "INSERT OR REPLACE INTO notes (filename, content, html_render, modified, is_indexed) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![filename, content, html_render, *fs_modified, true],
                )?;
            } else {
                tx.execute(
                    "INSERT OR REPLACE INTO notes (filename, content, html_render, modified, is_indexed) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![filename, content, "", *fs_modified, false],
                )?;
            }
        } else if !is_indexed && index < IMMEDIATE_RENDER_COUNT {
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

pub fn recreate_database() -> AppResult<()> {
    log(
        "DATABASE_RECREATE",
        "Database discrepancy detected - recreating tables",
        None,
    );

    with_db_mut(|conn| {
        conn.execute("DROP TABLE IF EXISTS notes", [])?;

        init_db(conn)?;

        load_all_notes_into_sqlite(conn)?;

        log(
            "DATABASE_RECREATE_SUCCESS",
            "Database recreated and synced from filesystem",
            None,
        );
        Ok(())
    })
}

pub async fn recreate_database_with_progress(
    app_handle: &AppHandle,
    reason: &str,
) -> AppResult<()> {
    if is_database_rebuilding() {
        log(
            "DATABASE_REBUILD_COLLISION",
            "Database rebuild already in progress - skipping duplicate rebuild",
            None,
        );
        return Err(AppError::DatabaseRebuild(
            "Database rebuild already in progress - please wait for completion".to_string(),
        ));
    }

    set_database_rebuilding(true);
    log(
        "DATABASE_REBUILD_START",
        "Database rebuild started - all database operations blocked",
        None,
    );

    if let Err(e) = app_handle.emit("db-loading-progress", "Rebuilding notes database...") {
        log(
            "UI_UPDATE",
            "Failed to emit rebuild progress",
            Some(&e.to_string()),
        );
    }
    eprintln!("{}", reason);

    let rebuild_result = with_db_mut(|conn| {
        conn.execute("DROP TABLE IF EXISTS notes", [])?;

        init_db(conn)?;

        if let Err(e) = app_handle.emit("db-loading-progress", "Rendering notes...") {
            log(
                "UI_UPDATE",
                "Failed to emit rendering progress",
                Some(&e.to_string()),
            );
        }

        load_all_notes_into_sqlite(conn).map_err(|e| e.into())
    });

    set_database_rebuilding(false);

    match rebuild_result {
        Ok(()) => {
            log(
                "DATABASE_REBUILD_SUCCESS",
                "Database rebuild completed successfully - database operations resumed",
                None,
            );
        }
        Err(ref e) => {
            log(
                "DATABASE_REBUILD_FAILURE",
                "Database rebuild failed - database operations resumed but may be inconsistent",
                Some(&e.to_string()),
            );
        }
    }

    if let Err(e) = app_handle.emit("db-loading-progress", "Notes database ready.") {
        log(
            "UI_UPDATE",
            "Failed to emit completion progress",
            Some(&e.to_string()),
        );
    }

    rebuild_result
}

pub fn quick_filesystem_sync_check() -> AppResult<bool> {
    let notes_dir = get_config_notes_dir();

    if !notes_dir.exists() {
        return Ok(true);
    }

    with_db(|conn| {
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

        if files.is_empty() {
            return Ok(true);
        }

        files.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));
        files.truncate(100);

        for entry in files {
            let file_path = entry.path();
            let relative_path = file_path.strip_prefix(&notes_dir).map_err(|e| {
                AppError::InvalidPath(format!("Failed to get relative path: {}", e))
            })?;
            let filename = relative_path.to_string_lossy().to_string();

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

            let file_modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let db_result: Result<(String, i64), rusqlite::Error> = conn.query_row(
                "SELECT content, modified FROM notes WHERE filename = ?1",
                params![filename],
                |row| Ok((row.get(0)?, row.get(1)?)),
            );

            match db_result {
                Ok((db_content, db_modified)) => {
                    if db_content != file_content {
                        return Ok(false);
                    }
                    if (db_modified - file_modified).abs() > 1 {
                        return Ok(false);
                    }
                }
                Err(_) => {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    })
}
