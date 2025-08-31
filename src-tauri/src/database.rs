// External crates
use crate::core::{AppError, AppResult};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

// Shared database connection manager
pub struct DatabaseManager {
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    pub fn new() -> AppResult<Self> {
        let db_path = get_database_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::DatabaseConnection(format!("Failed to create database directory: {}", e))
            })?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::DatabaseConnection(format!("Failed to open database: {}", e)))?;

        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn with_connection<T, F>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        let conn = self
            .connection
            .lock()
            .map_err(|e| AppError::DatabaseConnection(format!("Database lock poisoned: {}", e)))?;
        f(&*conn)
    }

    pub fn with_connection_mut<T, F>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut Connection) -> AppResult<T>,
    {
        let mut conn = self
            .connection
            .lock()
            .map_err(|e| AppError::DatabaseConnection(format!("Database lock poisoned: {}", e)))?;
        f(&mut *conn)
    }
}

// Global database manager instance
static DB_MANAGER: LazyLock<DatabaseManager> =
    LazyLock::new(|| DatabaseManager::new().expect("Failed to initialize database manager"));

pub fn with_db<T, F>(f: F) -> AppResult<T>
where
    F: FnOnce(&Connection) -> AppResult<T>,
{
    DB_MANAGER.with_connection(f)
}

pub fn with_db_mut<T, F>(f: F) -> AppResult<T>
where
    F: FnOnce(&mut Connection) -> AppResult<T>,
{
    DB_MANAGER.with_connection_mut(f)
}

pub fn get_database_path() -> AppResult<PathBuf> {
    let notes_dir = crate::config::get_config_notes_dir();
    get_database_path_for_notes_dir(&notes_dir)
}

pub fn get_database_path_for_notes_dir(notes_dir: &std::path::Path) -> AppResult<PathBuf> {
    let encoded_path = encode_path_for_backup(notes_dir);
    get_data_dir()
        .ok_or_else(|| AppError::ConfigLoad("Failed to get data directory".to_string()))
        .map(|path| {
            path.join("symiosis")
                .join("databases")
                .join(encoded_path)
                .join("notes.sqlite")
        })
}

fn encode_path_for_backup(notes_dir: &std::path::Path) -> String {
    notes_dir
        .to_string_lossy()
        .trim_start_matches('/') // Remove leading slash
        .trim_start_matches('\\') // Remove leading backslash (Windows)
        .replace('/', "_") // Replace slashes with underscores
        .replace('\\', "_") // Handle Windows backslashes
        .replace(':', "_") // Handle Windows drive letters (C:)
        .replace(' ', "_") // Replace spaces with underscores
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "_") // Replace other special chars
}

pub fn get_backup_dir_for_notes_path(notes_dir: &std::path::Path) -> AppResult<PathBuf> {
    let encoded_path = encode_path_for_backup(notes_dir);
    get_data_dir()
        .ok_or_else(|| AppError::ConfigLoad("Failed to get data directory".to_string()))
        .map(|path| path.join("symiosis").join("backups").join(encoded_path))
}

pub fn get_temp_dir() -> AppResult<PathBuf> {
    get_data_dir()
        .ok_or_else(|| AppError::ConfigLoad("Failed to get data directory".to_string()))
        .map(|path| path.join("symiosis").join("temp"))
}

// Platform-specific utility functions
pub fn get_data_dir() -> Option<PathBuf> {
    get_data_dir_impl()
}

fn get_data_dir_impl() -> Option<PathBuf> {
    if let Some(home_dir) = home::home_dir() {
        #[cfg(target_os = "macos")]
        return Some(home_dir.join("Library").join("Application Support"));

        #[cfg(target_os = "windows")]
        return std::env::var("APPDATA").ok().map(PathBuf::from);

        #[cfg(target_os = "linux")]
        return Some(home_dir.join(".local").join("share"));

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return Some(home_dir.join(".local").join("share"));
    }
    None
}
