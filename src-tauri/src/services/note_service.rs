use crate::{
    core::{AppError, AppResult},
    database::with_db,
    utilities::note_renderer::render_note,
};
use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn update_note_in_database(
    app_state: &crate::core::state::AppState,
    note_name: &str,
    content: &str,
    modified: i64,
) -> AppResult<()> {
    with_db(app_state, |conn| {
        // Generate HTML render from content
        let html_render = render_note(note_name, content);

        // First try to update existing note
        let updated_rows = conn
            .execute(
                "UPDATE notes SET content = ?2, html_render = ?3, modified = ?4, is_indexed = ?5 WHERE filename = ?1",
                params![note_name, content, html_render, modified, true],
            )?;

        // If no rows were updated, insert new note
        if updated_rows == 0 {
            conn.execute(
                "INSERT OR REPLACE INTO notes (filename, content, html_render, modified, is_indexed) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![note_name, content, html_render, modified, true],
            )?;
        }

        // Verify database was updated correctly
        let db_content = conn
            .query_row(
                "SELECT content FROM notes WHERE filename = ?1",
                params![note_name],
                |row| row.get::<_, String>(0),
            )
            .map_err(|e| {
                AppError::DatabaseQuery(format!("Failed to verify database update: {}", e))
            })?;

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
    })
}
