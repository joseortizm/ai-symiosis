use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use std::sync::LazyLock;

pub type DbPool = Pool<SqliteConnectionManager>;

static DB_POOL: LazyLock<DbPool> = LazyLock::new(|| {
    let db_path = get_database_path();
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let manager = SqliteConnectionManager::file(&db_path);
    Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("Failed to create database pool")
});

pub fn get_db_connection() -> Result<r2d2::PooledConnection<SqliteConnectionManager>, String> {
    DB_POOL
        .get()
        .map_err(|e| format!("Database pool error: {}", e))
}

fn get_database_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("./"))
        .join("symiosis")
        .join("notes.sqlite")
}
