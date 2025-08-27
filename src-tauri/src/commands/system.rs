use crate::{
    config::reload_config,
    database::get_db_connection,
    services::database_service::{
        init_db, load_all_notes_into_sqlite, load_all_notes_into_sqlite_with_progress,
        recreate_database_with_progress,
    },
    APP_CONFIG,
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn initialize_notes_with_progress(app: AppHandle) -> Result<(), String> {
    let result = async {
        std::thread::sleep(std::time::Duration::from_millis(50));

        let _ = app.emit("db-loading-start", "Initializing notes database...");

        if !crate::config::get_config_path().exists() {
            let _ = app.emit("db-loading-complete", ());
            return Ok(());
        }

        let _ = app.emit("db-loading-progress", "Setting up notes database...");

        let mut conn = match get_db_connection() {
            Ok(conn) => conn,
            Err(e) => {
                let error_msg = format!("Database connection error: {}", e);
                let _ = app.emit("db-loading-error", &error_msg);
                return Err(e);
            }
        };

        let _ = app.emit("db-loading-progress", "Loading notes from filesystem...");

        let app_clone = app.clone();
        let result = tokio::task::spawn_blocking(move || {
            load_all_notes_into_sqlite_with_progress(&mut conn, Some(&app_clone))
        })
        .await
        .map_err(|e| {
            crate::core::AppError::DatabaseConnection(format!("Task join error: {}", e))
        })?;

        match result {
            Ok(()) => {
                let _ = app.emit("db-loading-complete", ());
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to initialize notes database: {}", e);
                let _ = app.emit("db-loading-error", &error_msg);
                Err(e.into())
            }
        }
    }
    .await;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_cache(app: AppHandle) -> Result<(), String> {
    let result = async {
        let _ = app.emit("db-loading-start", "Refreshing notes...");
        let _ = app.emit("db-loading-progress", "Loading settings...");

        if let Err(e) = reload_config(&APP_CONFIG, Some(app.clone())) {
            let _ = app.emit(
                "db-loading-error",
                format!("Failed to reload config: {}", e),
            );
            return Err(crate::core::AppError::ConfigLoad(e));
        }

        let _ = app.emit("db-loading-progress", "Preparing notes database...");
        let mut conn = match get_db_connection() {
            Ok(conn) => conn,
            Err(e) => {
                let _ = app.emit(
                    "db-loading-error",
                    format!("Database connection error: {}", e),
                );
                return Err(e);
            }
        };

        let _ = app.emit("db-loading-progress", "Setting up notes database...");
        if let Err(e) = init_db(&conn) {
            let _ = app.emit(
                "db-loading-error",
                format!("Database initialization error: {}", e),
            );
            return Err(e.into());
        }

        let _ = app.emit("db-loading-progress", "Loading notes...");

        let result = tokio::task::spawn_blocking(move || load_all_notes_into_sqlite(&mut conn))
            .await
            .map_err(|e| {
                crate::core::AppError::DatabaseConnection(format!("Task join error: {}", e))
            })?;

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

                let result = recreate_database_with_progress(
                    &app,
                    "Database corruption detected. Recreating database tables...",
                )
                .await
                .map_err(|recovery_error| {
                    crate::core::AppError::DatabaseConnection(format!(
                        "Cache refresh failed and recovery failed: {}. Original error: {}",
                        recovery_error, e
                    ))
                });

                if result.is_ok() {
                    let _ = app.emit("db-loading-complete", ());
                } else if let Err(ref e) = result {
                    let _ = app.emit("db-loading-error", e.to_string());
                }
                Ok(result?)
            }
        }
    }
    .await;
    result.map_err(|e| e.to_string())
}
