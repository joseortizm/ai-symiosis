use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use std::sync::LazyLock;

pub type DbPool = Pool<SqliteConnectionManager>;

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

pub fn get_db_connection() -> Result<r2d2::PooledConnection<SqliteConnectionManager>, String> {
    match &*DB_POOL {
        Ok(pool) => pool
            .get()
            .map_err(|e| format!("Database pool error: {}", e)),
        Err(e) => Err(e.clone()),
    }
}

fn get_database_path() -> Result<PathBuf, String> {
    dirs::data_dir()
        .ok_or_else(|| "Failed to get data directory".to_string())
        .map(|path| path.join("symiosis").join("notes.sqlite"))
}
