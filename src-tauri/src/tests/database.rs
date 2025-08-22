//! Database Unit Tests
//!
//! Tests for database integration functionality and backup systems.

use crate::database::{get_backup_dir_for_notes_path, get_database_path, get_temp_dir};
use crate::*;

#[test]
fn test_database_path_creation() {
    let db_path = get_database_path().expect("Should get database path");
    assert!(db_path.is_absolute());
    assert!(db_path.to_string_lossy().contains("symiosis"));
    assert!(db_path.to_string_lossy().contains("notes.sqlite"));
}

#[test]
fn test_notes_directory_validation() {
    let notes_dir = get_config_notes_dir();
    assert!(notes_dir.is_absolute());
    assert!(!notes_dir.to_string_lossy().is_empty());
}

#[test]
fn test_backup_directory_creation() {
    let backup_dir = get_backup_dir_for_notes_path(&get_config_notes_dir())
        .expect("Should get backup directory");
    assert!(backup_dir.is_absolute());
    assert!(backup_dir.to_string_lossy().contains("symiosis"));
    assert!(backup_dir.to_string_lossy().contains("backups"));

    // Backup directory should be separate from notes directory
    let notes_dir = get_config_notes_dir();
    assert_ne!(
        backup_dir, notes_dir,
        "Backup directory should not be the same as notes directory"
    );
}

#[test]
fn test_temp_directory_creation() {
    let temp_dir = get_temp_dir().expect("Should get temp directory");
    assert!(temp_dir.is_absolute());
    assert!(temp_dir.to_string_lossy().contains("symiosis"));
    assert!(temp_dir.to_string_lossy().contains("temp"));

    // Temp directory should be separate from notes and backup directories
    let notes_dir = get_config_notes_dir();
    let backup_dir = get_backup_dir_for_notes_path(&get_config_notes_dir())
        .expect("Should get backup directory");
    assert_ne!(
        temp_dir, notes_dir,
        "Temp directory should not be the same as notes directory"
    );
    assert_ne!(
        temp_dir, backup_dir,
        "Temp directory should not be the same as backup directory"
    );
}

#[test]
fn test_directory_hierarchy() {
    let backup_dir = get_backup_dir_for_notes_path(&get_config_notes_dir())
        .expect("Should get backup directory");
    let temp_dir = get_temp_dir().expect("Should get temp directory");

    // Both should be under the same symiosis directory
    let backup_parent = backup_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Backup should have symiosis parent");
    let temp_parent = temp_dir.parent().expect("Temp should have parent");

    assert_eq!(
        backup_parent, temp_parent,
        "Backup and temp should share the same parent directory"
    );
    assert!(backup_parent.to_string_lossy().contains("symiosis"));
}

#[test]
fn test_path_specific_backup_directories() {
    use std::path::PathBuf;

    // Test different notes directories produce different backup paths
    let test_paths = vec![
        PathBuf::from("/Users/test/Documents/Notes"),
        PathBuf::from("/Users/test/Work/Notes"),
        PathBuf::from("/tmp/temporary-notes"),
        PathBuf::from("/home/user/notes with spaces"),
        PathBuf::from("C:\\Users\\Test\\Documents\\Notes"), // Windows path
    ];

    let mut backup_dirs = Vec::new();

    for path in &test_paths {
        let backup_dir = get_backup_dir_for_notes_path(path);
        assert!(
            backup_dir.is_ok(),
            "Should generate backup directory for path: {}",
            path.display()
        );
        backup_dirs.push(backup_dir.unwrap());
    }

    // All backup directories should be different
    for i in 0..backup_dirs.len() {
        for j in i + 1..backup_dirs.len() {
            assert_ne!(
                backup_dirs[i], backup_dirs[j],
                "Different notes paths should produce different backup directories"
            );
        }
    }

    // All should be under symiosis backup structure
    for (i, backup_dir) in backup_dirs.iter().enumerate() {
        assert!(
            backup_dir.is_absolute(),
            "Backup directory should be absolute"
        );
        assert!(
            backup_dir.to_string_lossy().contains("symiosis"),
            "Backup directory should contain 'symiosis': {}",
            backup_dir.display()
        );
        assert!(
            backup_dir.to_string_lossy().contains("backups"),
            "Backup directory should contain 'backups': {}",
            backup_dir.display()
        );

        // Should not equal the original notes directory
        assert_ne!(
            backup_dir, &test_paths[i],
            "Backup directory should not be the same as notes directory"
        );
    }
}

#[test]
fn test_path_encoding_for_backup_safety() {
    use std::path::PathBuf;

    // Test problematic paths are encoded safely
    let problematic_paths = vec![
        PathBuf::from("/path/with/slashes"),
        PathBuf::from("C:\\Windows\\Path\\With\\Backslashes"),
        PathBuf::from("/path with spaces/and-dashes"),
        PathBuf::from("/path:with:colons"),
        PathBuf::from("/path/with/special!@#$%^&*()characters"),
    ];

    for path in problematic_paths {
        let backup_dir = get_backup_dir_for_notes_path(&path);
        assert!(
            backup_dir.is_ok(),
            "Should handle problematic path: {}",
            path.display()
        );

        let backup_path = backup_dir.unwrap();
        let encoded_part = backup_path.file_name().unwrap().to_string_lossy();

        // Check that problematic characters are converted to safe alternatives
        assert!(
            !encoded_part.contains("/"),
            "Should not contain forward slashes"
        );
        assert!(
            !encoded_part.contains("\\"),
            "Should not contain backslashes"
        );
        assert!(!encoded_part.contains(":"), "Should not contain colons");
        assert!(!encoded_part.contains(" "), "Should not contain spaces");

        // Should only contain safe characters
        for c in encoded_part.chars() {
            assert!(
                c.is_alphanumeric() || c == '_',
                "Encoded path should only contain alphanumeric chars and underscores, found: '{}'",
                c
            );
        }
    }
}
