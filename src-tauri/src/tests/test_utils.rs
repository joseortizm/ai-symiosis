//! Test utilities module
//!
//! Contains test-only functions and structures that support testing
//! but are not part of the production codebase.

use crate::config::AppConfig;
use crate::services::database_service::recreate_database;
use crate::APP_CONFIG;
use std::sync::Mutex;
use tempfile::TempDir;

// Global mutex to prevent race conditions when multiple tests override config
#[cfg(test)]
static CONFIG_TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub mod database_testing;

/// Test utilities - provides isolated database connections for testing
pub struct DbTestHarness {
    _temp_dir: TempDir, // Keep alive for cleanup
    db_path: std::path::PathBuf,
}

impl DbTestHarness {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.sqlite");

        Ok(Self {
            _temp_dir: temp_dir,
            db_path,
        })
    }

    pub fn get_test_connection(&self) -> Result<rusqlite::Connection, String> {
        rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open test database: {}", e))
    }

    pub fn db_path(&self) -> &std::path::Path {
        &self.db_path
    }
}

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
        // Handle poisoned lock by taking ownership of the guard
        let lock = match CONFIG_TEST_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // Clear the poison and take the guard
                eprintln!("Warning: Test lock was poisoned, clearing and continuing");
                poisoned.into_inner()
            }
        };

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

        // Initialize a clean database for the test directory
        // Use recreate_database to ensure we start with a fresh database state
        recreate_database().map_err(|e| format!("Failed to recreate test database: {}", e))?;

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
