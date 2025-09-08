use crate::config::AppConfig;
use std::sync::{atomic::AtomicBool, Arc, RwLock};

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

    pub fn was_first_run(&self) -> &AtomicBool {
        &self.was_first_run
    }

    pub fn programmatic_operation_in_progress(&self) -> &AtomicBool {
        &self.programmatic_operation_in_progress
    }
}
