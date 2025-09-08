use crate::config::AppConfig;
use std::sync::{atomic::AtomicBool, Arc, Mutex, RwLock};

// Global pointer that shares the same Arc as AppState.config
// This eliminates dual state and ensures both managed and global access use the same config
static GLOBAL_CONFIG: std::sync::LazyLock<Mutex<Option<Arc<RwLock<AppConfig>>>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

// Global pointer to AppState for accessing atomic flags
static APP_STATE_HANDLE: std::sync::LazyLock<Mutex<Option<Arc<AppState>>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

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

/// Set the global state pointers to share the same Arc as the managed AppState
/// Call this once during app setup after .manage(AppState::new(...))
pub fn set_global_state(app_state: Arc<AppState>) {
    // Share the config Arc
    {
        let mut cfg_guard = GLOBAL_CONFIG.lock().unwrap();
        *cfg_guard = Some(Arc::clone(&app_state.config));
    }

    // Share the AppState Arc for atomic flag access
    {
        let mut state_guard = APP_STATE_HANDLE.lock().unwrap();
        *state_guard = Some(app_state);
    }
}

/// Global helper to set was_first_run flag
pub fn set_was_first_run(value: bool) {
    let state_opt = APP_STATE_HANDLE.lock().unwrap();
    if let Some(app_state) = state_opt.as_ref() {
        app_state
            .was_first_run
            .store(value, std::sync::atomic::Ordering::Relaxed);
    } else {
        #[cfg(not(test))]
        eprintln!("Warning: set_was_first_run called before set_global_state");
        // In test context, silently ignore - tests don't need the atomic flags
    }
}

/// Global helper to get was_first_run flag
pub fn get_was_first_run() -> bool {
    let state_opt = APP_STATE_HANDLE.lock().unwrap();
    if let Some(app_state) = state_opt.as_ref() {
        app_state
            .was_first_run
            .load(std::sync::atomic::Ordering::Relaxed)
    } else {
        #[cfg(not(test))]
        eprintln!("Warning: get_was_first_run called before set_global_state");
        false // Default to false in test context
    }
}

/// Global helper to set programmatic operation flag
pub fn set_programmatic_operation_in_progress(value: bool) {
    let state_opt = APP_STATE_HANDLE.lock().unwrap();
    if let Some(app_state) = state_opt.as_ref() {
        app_state
            .programmatic_operation_in_progress
            .store(value, std::sync::atomic::Ordering::Relaxed);
    } else {
        #[cfg(not(test))]
        eprintln!("Warning: set_programmatic_operation_in_progress called before set_global_state");
        // In test context, silently ignore - tests don't need the atomic flags
    }
}

/// Global helper to get programmatic operation flag
pub fn get_programmatic_operation_in_progress() -> bool {
    let state_opt = APP_STATE_HANDLE.lock().unwrap();
    if let Some(app_state) = state_opt.as_ref() {
        app_state
            .programmatic_operation_in_progress
            .load(std::sync::atomic::Ordering::Relaxed)
    } else {
        #[cfg(not(test))]
        eprintln!("Warning: get_programmatic_operation_in_progress called before set_global_state");
        false // Default to false in test context
    }
}

/// Centralized read access to config using callback pattern
/// Now uses the shared Arc from AppState - no duplication!
pub fn with_config<F, R>(f: F) -> R
where
    F: FnOnce(&AppConfig) -> R,
{
    let arc = {
        let cfg_opt = GLOBAL_CONFIG.lock().unwrap();
        match cfg_opt.as_ref() {
            Some(arc) => arc.clone(),
            None => {
                #[cfg(test)]
                {
                    // In test context, create a temporary config
                    use crate::config::load_config;
                    let test_config = Arc::new(RwLock::new(load_config()));
                    drop(cfg_opt); // Release the lock
                    let mut cfg_guard = GLOBAL_CONFIG.lock().unwrap();
                    *cfg_guard = Some(test_config.clone());
                    test_config
                }
                #[cfg(not(test))]
                {
                    panic!("GLOBAL_CONFIG not initialized; ensure set_global_state() is called in setup");
                }
            }
        }
    };
    let guard = arc.read().unwrap_or_else(|e| e.into_inner());
    f(&*guard)
}

/// Centralized write access to config using callback pattern
/// Now uses the shared Arc from AppState - no duplication!
pub fn with_config_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut AppConfig) -> R,
{
    let arc = {
        let cfg_opt = GLOBAL_CONFIG.lock().unwrap();
        match cfg_opt.as_ref() {
            Some(arc) => arc.clone(),
            None => {
                #[cfg(test)]
                {
                    // In test context, create a temporary config
                    use crate::config::load_config;
                    let test_config = Arc::new(RwLock::new(load_config()));
                    drop(cfg_opt); // Release the lock
                    let mut cfg_guard = GLOBAL_CONFIG.lock().unwrap();
                    *cfg_guard = Some(test_config.clone());
                    test_config
                }
                #[cfg(not(test))]
                {
                    panic!("GLOBAL_CONFIG not initialized; ensure set_global_state() is called in setup");
                }
            }
        }
    };
    let mut guard = arc.write().unwrap_or_else(|e| e.into_inner());
    f(&mut *guard)
}

/// Execute function that needs RwLock reference (for functions like reload_config)
/// Now uses the shared Arc from AppState - no duplication!
pub fn with_config_lock<F, R>(f: F) -> R
where
    F: FnOnce(&std::sync::RwLock<AppConfig>) -> R,
{
    let arc = {
        let cfg_opt = GLOBAL_CONFIG.lock().unwrap();
        match cfg_opt.as_ref() {
            Some(arc) => arc.clone(),
            None => {
                #[cfg(test)]
                {
                    // In test context, create a temporary config
                    use crate::config::load_config;
                    let test_config = Arc::new(RwLock::new(load_config()));
                    drop(cfg_opt); // Release the lock
                    let mut cfg_guard = GLOBAL_CONFIG.lock().unwrap();
                    *cfg_guard = Some(test_config.clone());
                    test_config
                }
                #[cfg(not(test))]
                {
                    panic!("GLOBAL_CONFIG not initialized; ensure set_global_state() is called in setup");
                }
            }
        }
    };
    f(&*arc)
}
