use crate::{config::AppConfig, core::AppResult, database::DatabaseManager};
use std::sync::{atomic::AtomicBool, Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub was_first_run: Arc<AtomicBool>,
    pub programmatic_operation_in_progress: Arc<AtomicBool>,
    pub database_manager: Arc<Mutex<DatabaseManager>>,
    pub database_lock: Arc<Mutex<()>>,
    pub database_rebuild_lock: Arc<RwLock<bool>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> AppResult<Self> {
        let database_manager = DatabaseManager::new()?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            was_first_run: Arc::new(AtomicBool::new(false)),
            programmatic_operation_in_progress: Arc::new(AtomicBool::new(false)),
            database_manager: Arc::new(Mutex::new(database_manager)),
            database_lock: Arc::new(Mutex::new(())),
            database_rebuild_lock: Arc::new(RwLock::new(false)),
        })
    }

    pub fn new_with_fallback(config: AppConfig) -> Self {
        match Self::new(config.clone()) {
            Ok(state) => state,
            Err(_) => {
                // Create fallback state that allows app to continue running
                // Database operations will gracefully handle the failed state
                Self {
                    config: Arc::new(RwLock::new(config)),
                    was_first_run: Arc::new(AtomicBool::new(false)),
                    programmatic_operation_in_progress: Arc::new(AtomicBool::new(false)),
                    database_manager: Arc::new(Mutex::new(DatabaseManager::new_fallback())),
                    database_lock: Arc::new(Mutex::new(())),
                    database_rebuild_lock: Arc::new(RwLock::new(false)),
                }
            }
        }
    }

    pub fn set_first_run(&self, value: bool) {
        self.was_first_run
            .store(value, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn was_first_run(&self) -> &AtomicBool {
        &self.was_first_run
    }

    pub fn programmatic_operation_in_progress(&self) -> &AtomicBool {
        &self.programmatic_operation_in_progress
    }

    pub fn is_database_rebuilding(&self) -> AppResult<bool> {
        Ok(*self.database_rebuild_lock.read().map_err(|e| {
            crate::core::AppError::DatabaseConnection(format!(
                "Database rebuild lock poisoned: {}",
                e
            ))
        })?)
    }

    pub fn set_database_rebuilding(&self, rebuilding: bool) -> AppResult<()> {
        *self.database_rebuild_lock.write().map_err(|e| {
            crate::core::AppError::DatabaseConnection(format!(
                "Database rebuild lock poisoned: {}",
                e
            ))
        })? = rebuilding;
        Ok(())
    }
}
