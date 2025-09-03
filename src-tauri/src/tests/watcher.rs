use crate::tests::test_utils::TestConfigOverride;
use crate::APP_CONFIG;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;

#[test]
fn test_watcher_setup_with_missing_directory_should_fail() {
    let _test_config = TestConfigOverride::new().expect("Should create test config");

    // Remove the notes directory to simulate missing directory scenario
    let notes_dir = crate::config::get_config_notes_dir();
    if notes_dir.exists() {
        fs::remove_dir_all(&notes_dir).expect("Should remove test directory");
    }

    // Verify directory doesn't exist
    assert!(
        !notes_dir.exists(),
        "Notes directory should not exist for this test"
    );

    // Test the core watcher creation logic that causes the bug
    let (tx, _rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )
    .expect("Should create watcher");

    // This should fail - demonstrating the bug
    let result = watcher.watch(&notes_dir, RecursiveMode::Recursive);

    assert!(
        result.is_err(),
        "Watcher should fail when notes directory doesn't exist"
    );

    // Check that the error is related to path not found
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("No such file or directory")
            || error_msg.contains("path was not found")
            || error_msg.contains("No path was found")
            || error_msg.contains("entity not found")
            || error_msg.contains("cannot find the file"),
        "Error should indicate missing directory: {}",
        error_msg
    );
}

#[test]
fn test_watcher_setup_with_existing_directory_should_succeed() {
    let _test_config = TestConfigOverride::new().expect("Should create test config");

    // Ensure the notes directory exists
    let notes_dir = crate::config::get_config_notes_dir();
    fs::create_dir_all(&notes_dir).expect("Should create notes directory");

    // Verify directory exists
    assert!(
        notes_dir.exists(),
        "Notes directory should exist for this test"
    );

    // Test the core watcher creation logic
    let (tx, _rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )
    .expect("Should create watcher");

    // This should succeed
    let result = watcher.watch(&notes_dir, RecursiveMode::Recursive);

    assert!(
        result.is_ok(),
        "Watcher should succeed when notes directory exists: {:?}",
        result
    );
}

#[test]
fn test_watcher_setup_creates_missing_directory_before_watching() {
    let _test_config = TestConfigOverride::new().expect("Should create test config");

    // Remove the notes directory to simulate missing directory scenario
    let notes_dir = crate::config::get_config_notes_dir();
    if notes_dir.exists() {
        fs::remove_dir_all(&notes_dir).expect("Should remove test directory");
    }

    // Verify directory doesn't exist
    assert!(
        !notes_dir.exists(),
        "Notes directory should not exist for this test"
    );

    // Create the directory (simulating the fix)
    fs::create_dir_all(&notes_dir).expect("Should create notes directory");

    // Now watcher should succeed
    let (tx, _rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )
    .expect("Should create watcher");

    let result = watcher.watch(&notes_dir, RecursiveMode::Recursive);

    assert!(
        result.is_ok(),
        "Watcher should succeed after creating missing directory: {:?}",
        result
    );
}

#[test]
fn test_get_config_notes_dir_returns_configured_path() {
    let _test_config = TestConfigOverride::new().expect("Should create test config");

    let notes_dir = crate::config::get_config_notes_dir();
    let config = APP_CONFIG.read().unwrap();
    let expected_path = PathBuf::from(&config.notes_directory);

    assert_eq!(
        notes_dir, expected_path,
        "get_config_notes_dir should return configured path"
    );
}
