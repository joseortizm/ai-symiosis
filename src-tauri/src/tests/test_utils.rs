//! Test utilities module
//!
//! Contains test-only functions and structures that support testing
//! but are not part of the production codebase.

use crate::config::AppConfig;
use crate::core::state::with_config_lock;
use crate::services::database_service::recreate_database;
use std::path::Path;
use std::sync::Mutex;
use tempfile::TempDir;

// Global mutex to prevent race conditions when multiple tests override config
#[cfg(test)]
static CONFIG_TEST_LOCK: Mutex<()> = Mutex::new(());

/// CRITICAL SAFETY: Validate that a directory path is safe for test usage
/// This prevents accidental data loss by ensuring tests only use approved directories
#[cfg(test)]
fn validate_test_directory_safety(path: &Path) -> Result<(), String> {
    let path_str = path.to_string_lossy();

    // SAFETY: Only allow specific patterns that indicate test directories
    let is_safe_test_dir =
        // Temporary directories created by tempfile crate
        path_str.contains("/tmp/") ||
        path_str.contains("\\Temp\\") ||
        // Explicit test markers in path
        path_str.contains("_tmp") ||
        path_str.contains("test") ||
        path_str.contains("Test") ||
        // System temp directories
        path_str.starts_with("/var/folders/") ||  // macOS temp
        path_str.starts_with("/tmp/") ||           // Unix temp
        path_str.contains("/AppData/Local/Temp/") || // Windows temp
        // CI/CD environments
        std::env::var("CI").is_ok();

    if !is_safe_test_dir {
        return Err(format!(
            "CRITICAL SAFETY ERROR: Directory '{}' is not recognized as a safe test directory. \
             Tests can only use temporary directories or paths containing 'test'/'_tmp' markers. \
             This prevents accidental deletion of production data.",
            path_str
        ));
    }

    // Additional safety: Check for common production directory patterns
    let dangerous_patterns = [
        "Documents",
        "Desktop",
        "Downloads",
        "Pictures",
        "Videos",
        "Music",
        "notes",
        "Notes",
        "notebook",
        "Notebook",
        ".git",
        "src",
        "home",
        "Users",
        "user",
    ];

    // Only flag as dangerous if it's NOT in a temp directory
    if !path_str.contains("/tmp/")
        && !path_str.contains("\\Temp\\")
        && !path_str.starts_with("/var/folders/")
    {
        for pattern in &dangerous_patterns {
            if path_str.contains(pattern) {
                return Err(format!(
                    "CRITICAL SAFETY ERROR: Directory '{}' contains suspicious pattern '{}' \
                     and is not in a recognized temp location. This could indicate a production directory.",
                    path_str, pattern
                ));
            }
        }
    }

    Ok(())
}

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

        // CRITICAL SAFETY: Validate the temporary directory is safe for testing
        validate_test_directory_safety(temp_dir.path())
            .map_err(|e| format!("Test safety validation failed: {}", e))?;

        let test_notes_path = temp_dir.path().to_string_lossy().to_string();

        let original_config = with_config_lock(|config_lock| {
            config_lock
                .read()
                .map_err(|e| format!("Failed to read config: {}", e))
                .map(|guard| guard.clone())
        })?;

        // Create a new config with the test directory
        let mut test_config = original_config.clone();
        test_config.notes_directory = test_notes_path;

        with_config_lock(|config_lock| -> Result<(), String> {
            match config_lock.write() {
                Ok(mut guard) => {
                    *guard = test_config;
                    Ok(())
                }
                Err(e) => Err(format!("Failed to write config: {}", e)),
            }
        })?;

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
        let _ = with_config_lock(|config_lock| {
            if let Ok(mut guard) = config_lock.write() {
                *guard = self.original_config.clone();
            }
        });
    }
}

/// Test wrapper functions using tauri::test::mock_app()
/// These use Tauri's official testing utilities to properly mock State
#[cfg(test)]
mod test_command_wrappers {
    use crate::core::state::with_config_lock;
    use crate::core::state::AppState;
    use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
    use tauri::{App, Manager};

    /// Create a mock Tauri app with test AppState
    fn create_test_mock_app() -> App<MockRuntime> {
        let config = with_config_lock(|config_lock| {
            config_lock
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .clone()
        });

        let app_state = AppState::new(config);

        mock_builder()
            .manage(app_state)
            .build(mock_context(noop_assets()))
            .expect("Failed to build test app")
    }

    pub fn test_create_new_note(note_name: &str) -> Result<(), String> {
        let app = create_test_mock_app();
        let app_state = app.state::<AppState>();
        crate::commands::notes::create_new_note(note_name, app_state)
    }

    pub fn test_get_note_content(note_name: &str) -> Result<String, String> {
        crate::commands::notes::get_note_content(note_name)
    }

    pub fn test_delete_note(note_name: &str) -> Result<(), String> {
        let app = create_test_mock_app();
        let app_state = app.state::<AppState>();
        crate::commands::notes::delete_note(note_name, app_state)
    }

    pub fn test_save_note_with_content_check(
        note_name: &str,
        content: &str,
        original_content: &str,
    ) -> Result<(), String> {
        let app = create_test_mock_app();
        let app_state = app.state::<AppState>();
        crate::commands::notes::save_note_with_content_check(
            note_name,
            content,
            original_content,
            app_state,
        )
    }

    pub fn test_rename_note(old_name: String, new_name: String) -> Result<(), String> {
        let app = create_test_mock_app();
        let app_state = app.state::<AppState>();
        crate::commands::notes::rename_note(old_name, new_name, app_state)
    }
}

#[cfg(test)]
pub use test_command_wrappers::*;
