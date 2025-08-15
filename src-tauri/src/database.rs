// External crates
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use std::sync::LazyLock;

// Type definitions
pub type DbPool = Pool<SqliteConnectionManager>;

// Global database pool
static DB_POOL: LazyLock<Result<DbPool, String>> = LazyLock::new(|| {
    let db_path = get_database_path()?;
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create database directory: {}", e))?;
    }

    let manager = SqliteConnectionManager::file(&db_path);
    Pool::builder()
        .max_size(10)
        .build(manager)
        .map_err(|e| format!("Failed to create database pool: {}", e))
});

// Public API functions
pub fn get_db_connection() -> Result<r2d2::PooledConnection<SqliteConnectionManager>, String> {
    match &*DB_POOL {
        Ok(pool) => pool
            .get()
            .map_err(|e| format!("Database pool error: {}", e)),
        Err(e) => Err(e.clone()),
    }
}

pub fn get_database_path() -> Result<PathBuf, String> {
    get_data_dir()
        .ok_or_else(|| "Failed to get data directory".to_string())
        .map(|path| path.join("symiosis").join("notes.sqlite"))
}

// Platform-specific utility functions
fn get_data_dir() -> Option<PathBuf> {
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
