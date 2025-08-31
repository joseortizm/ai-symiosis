//! Test utilities module
//!
//! Contains test-only functions and structures that support testing
//! but are not part of the production codebase.

use crate::config::AppConfig;
use crate::database::with_db;
use crate::services::database_service::init_db;
use crate::APP_CONFIG;
use std::sync::Mutex;
use tempfile::TempDir;

// Global mutex to prevent race conditions when multiple tests override config
#[cfg(test)]
static CONFIG_TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub mod database_testing;

/// Test configuration override utility
///
/// This struct temporarily overrides the global APP_CONFIG to use a test directory,
/// ensuring all production functions automatically work with isolated test data.
#[cfg(test)]
pub struct TestConfigOverride {
    _temp_dir: TempDir,
    original_config: AppConfig,
    _lock: std::sync::MutexGuard<'static, ()>,
}

#[cfg(test)]
impl TestConfigOverride {
    /// Create a new test config override with an isolated temporary directory
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Acquire lock to prevent race conditions between parallel tests
        let lock = CONFIG_TEST_LOCK
            .lock()
            .map_err(|e| format!("Failed to acquire test lock: {}", e))?;

        let temp_dir = TempDir::new()?;
        let test_notes_path = temp_dir.path().to_string_lossy().to_string();

        // Read the current config and store it for restoration
        let original_config = {
            let config_guard = APP_CONFIG
                .read()
                .map_err(|e| format!("Failed to read config: {}", e))?;
            config_guard.clone()
        };

        // Create a new config with the test directory
        let mut test_config = original_config.clone();
        test_config.notes_directory = test_notes_path;

        // Override the global config
        {
            let mut config_guard = APP_CONFIG
                .write()
                .map_err(|e| format!("Failed to write config: {}", e))?;
            *config_guard = test_config;
        }

        // Initialize the database for the test directory
        with_db(|conn| init_db(conn).map_err(crate::core::AppError::from))
            .map_err(|e| format!("Failed to initialize test database: {}", e))?;

        Ok(Self {
            _temp_dir: temp_dir,
            original_config,
            _lock: lock,
        })
    }

    /// Get the temporary notes directory path
    pub fn notes_dir(&self) -> std::path::PathBuf {
        self._temp_dir.path().to_path_buf()
    }
}

#[cfg(test)]
impl Drop for TestConfigOverride {
    fn drop(&mut self) {
        // Restore the original configuration
        if let Ok(mut config_guard) = APP_CONFIG.write() {
            *config_guard = self.original_config.clone();
        }
    }
}
