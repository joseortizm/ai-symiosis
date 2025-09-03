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

/// Clean up all _tmp* directories (removes leftover test directories)
#[cfg(test)]
pub fn cleanup_all_tmp_directories() -> Result<(), Box<dyn std::error::Error>> {
    use crate::database::get_data_dir;
    use std::fs;

    // Get the symiosis app support directory using the same method as production code
    if let Some(app_data_dir) = get_data_dir() {
        let symiosis_dir = app_data_dir.join("symiosis");

        // Clean up databases directory
        let databases_dir = symiosis_dir.join("databases");
        if databases_dir.exists() {
            // Remove all _tmp* directories
            if let Ok(entries) = fs::read_dir(&databases_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                            if dir_name.starts_with("_tmp") {
                                let _ = fs::remove_dir_all(&path);
                            }
                        }
                    }
                }
            }
        }

        // Clean up backups directory
        let backups_dir = symiosis_dir.join("backups");
        if backups_dir.exists() {
            // Remove all _tmp* directories
            if let Ok(entries) = fs::read_dir(&backups_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                            if dir_name.starts_with("_tmp") {
                                let _ = fs::remove_dir_all(&path);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

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
}

/// Test configuration override utility
///
/// This struct temporarily overrides the global APP_CONFIG to use a test directory,
/// ensuring all production functions automatically work with isolated test data.
/// It tracks and cleans up database and backup directories created during tests.
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

        // Don't auto-cleanup during Drop to avoid database lock issues
        // The directories are tracked and can be cleaned up manually
        // The TempDir will still clean up the notes directory automatically
    }
}
