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

pub(crate) fn encode_path_for_backup(notes_dir: &std::path::Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Get the last component of the path for a friendly name
    let friendly_name = notes_dir
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("notes"))
        .to_string_lossy()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    // Hash the full absolute path to guarantee uniqueness
    let mut hasher = DefaultHasher::new();
    notes_dir.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();

    // Create short hash (6 hex chars should be enough for uniqueness)
    let short_hash = format!("{:06x}", hash & 0xFFFFFF);

    // Combine friendly name with hash: "notes-3f8c9a"
    format!("{}-{}", friendly_name, short_hash)
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
