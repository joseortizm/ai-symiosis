use crate::core::{AppError, AppResult};
use crate::utilities::paths::get_database_path;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct DatabaseManager {
    connection: Connection,
    current_db_path: PathBuf,
}

impl DatabaseManager {
    pub fn new() -> AppResult<Self> {
        let db_path = get_database_path()?;
        let conn = Self::create_connection(&db_path)?;

        Ok(Self {
            connection: conn,
            current_db_path: db_path,
        })
    }

    fn create_connection(db_path: &PathBuf) -> AppResult<Connection> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::DatabaseConnection(format!("Failed to create database directory: {}", e))
            })?;
        }

        Connection::open(db_path)
            .map_err(|e| AppError::DatabaseConnection(format!("Failed to open database: {}", e)))
    }

    pub fn ensure_current_connection(&mut self) -> AppResult<bool> {
        let expected_db_path = get_database_path()?;

        if self.current_db_path != expected_db_path {
            let new_conn = Self::create_connection(&expected_db_path)?;
            // Atomically replace both connection and path
            self.connection = new_conn;
            self.current_db_path = expected_db_path;
            Ok(true) // Connection was reinitialized
        } else {
            Ok(false) // No reinitialization needed
        }
    }

    pub fn with_connection<T, F>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        f(&self.connection)
    }

    pub fn with_connection_mut<T, F>(&mut self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut Connection) -> AppResult<T>,
    {
        f(&mut self.connection)
    }
}

pub fn with_db<T, F>(app_state: &crate::core::state::AppState, f: F) -> AppResult<T>
where
    F: FnOnce(&Connection) -> AppResult<T>,
{
    // First acquire read lock on rebuild_lock to ensure no rebuilds are happening
    let _rebuild_guard = app_state.database_rebuild_lock.read().map_err(|e| {
        AppError::DatabaseConnection(format!("Database rebuild lock poisoned: {}", e))
    })?;

    // Then acquire database manager lock
    let manager = app_state.database_manager.lock().map_err(|e| {
        AppError::DatabaseConnection(format!("Database manager lock poisoned: {}", e))
    })?;

    manager.with_connection(f)
}

pub fn with_db_mut<T, F>(app_state: &crate::core::state::AppState, f: F) -> AppResult<T>
where
    F: FnOnce(&mut Connection) -> AppResult<T>,
{
    // First acquire read lock on rebuild_lock to ensure no rebuilds are happening
    let _rebuild_guard = app_state.database_rebuild_lock.read().map_err(|e| {
        AppError::DatabaseConnection(format!("Database rebuild lock poisoned: {}", e))
    })?;

    // Then acquire database manager lock
    let mut manager = app_state.database_manager.lock().map_err(|e| {
        AppError::DatabaseConnection(format!("Database manager lock poisoned: {}", e))
    })?;

    manager.with_connection_mut(f)
}

pub fn refresh_database_connection(app_state: &crate::core::state::AppState) -> AppResult<bool> {
    // First acquire read lock on rebuild_lock to ensure no rebuilds are happening
    let _rebuild_guard = app_state.database_rebuild_lock.read().map_err(|e| {
        AppError::DatabaseConnection(format!("Database rebuild lock poisoned: {}", e))
    })?;

    // Then acquire database manager lock
    let mut manager = app_state.database_manager.lock().map_err(|e| {
        AppError::DatabaseConnection(format!("Database manager lock poisoned: {}", e))
    })?;

    manager.ensure_current_connection()
}

// Platform-specific utility functions
