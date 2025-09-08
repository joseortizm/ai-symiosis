use crate::config::{load_config, AppConfig};
use std::sync::{atomic::AtomicBool, Arc, LazyLock, RwLock};

// MIGRATION: Temporary location for APP_CONFIG during migration
pub static APP_CONFIG: LazyLock<RwLock<AppConfig>> = LazyLock::new(|| RwLock::new(load_config()));

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub was_first_run: Arc<AtomicBool>,
    pub programmatic_operation_in_progress: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            was_first_run: Arc::new(AtomicBool::new(false)),
            programmatic_operation_in_progress: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_first_run(&self, value: bool) {
        self.was_first_run
            .store(value, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_config(&self) -> Arc<RwLock<AppConfig>> {
        Arc::clone(&self.config)
    }

    pub fn was_first_run(&self) -> &AtomicBool {
        &self.was_first_run
    }

    pub fn programmatic_operation_in_progress(&self) -> &AtomicBool {
        &self.programmatic_operation_in_progress
    }
}

/// Centralized read access to config using callback pattern
pub fn with_config<F, R>(f: F) -> R
where
    F: FnOnce(&AppConfig) -> R,
{
    let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    f(&config)

    // Future implementation after migration:
    // let config_arc = app_state.get_config();
    // let config = config_arc.read().unwrap_or_else(|e| e.into_inner());
    // f(&config)
}

/// Centralized write access to config using callback pattern
pub fn with_config_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut AppConfig) -> R,
{
    let mut config = APP_CONFIG.write().unwrap_or_else(|e| e.into_inner());
    f(&mut config)

    // Future implementation after migration:
    // let config_arc = app_state.get_config();
    // let mut config = config_arc.write().unwrap_or_else(|e| e.into_inner());
    // f(&mut config)
}

/// Execute function that needs RwLock reference (for functions like reload_config)
pub fn with_config_lock<F, R>(f: F) -> R
where
    F: FnOnce(&std::sync::RwLock<AppConfig>) -> R,
{
    f(&APP_CONFIG)

    // Future implementation after migration:
    // let config_arc = app_state.get_config();
    // f(&*config_arc)
}
