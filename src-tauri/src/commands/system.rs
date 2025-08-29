use crate::{
    config::reload_config,
    database::with_db_mut,
    logging::log,
    services::database_service::{
        init_db, load_all_notes_into_sqlite, load_all_notes_into_sqlite_with_progress,
        recreate_database_with_progress,
    },
    APP_CONFIG,
};
use tauri::{AppHandle, Emitter};

fn emit_with_logging<T: serde::Serialize + Clone>(app: &AppHandle, event: &str, payload: T) {
    if let Err(e) = app.emit(event, payload) {
        log(
            "UI_UPDATE",
            &format!("Failed to emit {}", event),
            Some(&e.to_string()),
        );
    }
}

#[tauri::command]
pub async fn initialize_notes_with_progress(app: AppHandle) -> Result<(), String> {
    let result = async {
        std::thread::sleep(std::time::Duration::from_millis(50));

        emit_with_logging(&app, "db-loading-start", "Initializing notes database...");

        if !crate::config::get_config_path().exists() {
            emit_with_logging(&app, "db-loading-complete", ());
            return Ok(());
        }

        emit_with_logging(&app, "db-loading-progress", "Setting up notes database...");

        emit_with_logging(
            &app,
            "db-loading-progress",
            "Loading notes from filesystem...",
        );

        let app_clone = app.clone();
        let result = tokio::task::spawn_blocking(move || {
            with_db_mut(|conn| {
                load_all_notes_into_sqlite_with_progress(conn, Some(&app_clone))
                    .map_err(|e| e.into())
            })
        })
        .await
        .map_err(|e| {
            crate::core::AppError::DatabaseConnection(format!("Task join error: {}", e))
        })?;

        match result {
            Ok(()) => {
                emit_with_logging(&app, "db-loading-complete", ());
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to initialize notes database: {}", e);
                emit_with_logging(&app, "db-loading-error", &error_msg);
                Err(e.into())
            }
        }
    }
    .await;
    result.map_err(|e: crate::core::AppError| e.to_string())
}

#[tauri::command]
pub async fn refresh_cache(app: AppHandle) -> Result<(), String> {
    let result = async {
        emit_with_logging(&app, "db-loading-start", "Refreshing notes...");
        emit_with_logging(&app, "db-loading-progress", "Loading settings...");

        if let Err(e) = reload_config(&APP_CONFIG, Some(app.clone())) {
            emit_with_logging(
                &app,
                "db-loading-error",
                format!("Failed to reload config: {}", e),
            );
            return Err(crate::core::AppError::ConfigLoad(e));
        }

        emit_with_logging(&app, "db-loading-progress", "Preparing notes database...");

        emit_with_logging(&app, "db-loading-progress", "Setting up notes database...");

        emit_with_logging(&app, "db-loading-progress", "Loading notes...");

        let result = tokio::task::spawn_blocking(move || {
            with_db_mut(|conn| {
                init_db(conn)?;
                load_all_notes_into_sqlite(conn).map_err(|e| e.into())
            })
        })
        .await
        .map_err(|e| {
            crate::core::AppError::DatabaseConnection(format!("Task join error: {}", e))
        })?;

        match result {
            Ok(()) => {
                emit_with_logging(&app, "db-loading-complete", ());
                Ok(())
            }
            Err(e) => {
                emit_with_logging(
                    &app,
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
                    emit_with_logging(&app, "db-loading-complete", ());
                } else if let Err(ref e) = result {
                    emit_with_logging(&app, "db-loading-error", e.to_string());
                }
                Ok(result?)
            }
        }
    }
    .await;
    result.map_err(|e: crate::core::AppError| e.to_string())
}
