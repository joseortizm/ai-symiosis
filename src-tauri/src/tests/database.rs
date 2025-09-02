//! Database Unit Tests
//!
//! Tests for database integration functionality and backup systems.

use crate::config::get_config_notes_dir;
use crate::database::{
    encode_path_for_backup, get_backup_dir_for_notes_path, get_database_path_for_notes_dir,
    get_temp_dir,
};
use std::path::PathBuf;

#[test]
fn test_database_path_creation_logic() {
    // Test the path generation logic without creating actual directories
    let test_notes_dir = PathBuf::from("/test/notes/directory");
    let db_path =
        get_database_path_for_notes_dir(&test_notes_dir).expect("Should generate database path");

    assert!(db_path.is_absolute());
    assert!(db_path.to_string_lossy().contains("symiosis"));
    assert!(db_path.to_string_lossy().contains("databases"));
    assert!(db_path.to_string_lossy().contains("notes.sqlite"));

    // Should contain encoded version of the test path
    let encoded_part = encode_path_for_backup(&test_notes_dir);
    assert!(db_path.to_string_lossy().contains(&encoded_part));
}

#[test]
fn test_notes_directory_validation() {
    let notes_dir = get_config_notes_dir();
    assert!(notes_dir.is_absolute());
    assert!(!notes_dir.to_string_lossy().is_empty());
}

#[test]
fn test_backup_directory_creation_logic() {
    // Test backup directory path generation without creating actual directories
    let test_notes_dir = PathBuf::from("/test/notes/directory");
    let backup_dir = get_backup_dir_for_notes_path(&test_notes_dir)
        .expect("Should generate backup directory path");

    assert!(backup_dir.is_absolute());
    assert!(backup_dir.to_string_lossy().contains("symiosis"));
    assert!(backup_dir.to_string_lossy().contains("backups"));

    // Should contain encoded version of the test path
    let encoded_part = encode_path_for_backup(&test_notes_dir);
    assert!(backup_dir.to_string_lossy().contains(&encoded_part));

    // Backup directory should be separate from notes directory
    assert_ne!(
        backup_dir, test_notes_dir,
        "Backup directory should not be the same as notes directory"
    );
}

#[test]
fn test_temp_directory_creation_logic() {
    // Test temp directory path generation logic
    let temp_dir = get_temp_dir().expect("Should generate temp directory path");
    assert!(temp_dir.is_absolute());
    assert!(temp_dir.to_string_lossy().contains("symiosis"));
    assert!(temp_dir.to_string_lossy().contains("temp"));

    // Test that different directory types generate different paths
    let test_notes_dir = PathBuf::from("/test/notes/directory");
    let backup_dir = get_backup_dir_for_notes_path(&test_notes_dir)
        .expect("Should generate backup directory path");

    assert_ne!(
        temp_dir, test_notes_dir,
        "Temp directory should not be the same as notes directory"
    );
    assert_ne!(
        temp_dir, backup_dir,
        "Temp directory should not be the same as backup directory"
    );
}

#[test]
fn test_directory_hierarchy_logic() {
    // Test directory hierarchy without creating real directories
    let test_notes_dir = PathBuf::from("/test/notes/directory");
    let backup_dir = get_backup_dir_for_notes_path(&test_notes_dir)
        .expect("Should generate backup directory path");
    let temp_dir = get_temp_dir().expect("Should generate temp directory path");

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

        // Should only contain safe characters (alphanumeric, underscore, dash)
        for c in encoded_part.chars() {
            assert!(
                c.is_alphanumeric() || c == '_' || c == '-',
                "Encoded path should only contain alphanumeric chars, underscores, and dashes, found: '{}'",
                c
            );
        }
    }
}

#[test]
fn test_path_encoding_with_friendly_names_and_uniqueness() {
    use std::path::Path;

    // Test that different paths produce different encoded strings
    let path1 = Path::new("/home/user/notes");
    let path2 = Path::new("/home:user:notes");
    let path3 = Path::new("/home user notes");
    let path4 = Path::new("/Documents/_notes");
    let path5 = Path::new("/Users/alice/Documents/_notes");

    let encoded1 = encode_path_for_backup(path1);
    let encoded2 = encode_path_for_backup(path2);
    let encoded3 = encode_path_for_backup(path3);
    let encoded4 = encode_path_for_backup(path4);
    let encoded5 = encode_path_for_backup(path5);

    println!("EXPECTED BEHAVIOR:");
    println!("Path 1: {:?} -> {}", path1, encoded1);
    println!("Path 2: {:?} -> {}", path2, encoded2);
    println!("Path 3: {:?} -> {}", path3, encoded3);
    println!("Path 4: {:?} -> {}", path4, encoded4);
    println!("Path 5: {:?} -> {}", path5, encoded5);

    // All encoded strings should be different (no collisions)
    let encodings = vec![&encoded1, &encoded2, &encoded3, &encoded4, &encoded5];
    for i in 0..encodings.len() {
        for j in i + 1..encodings.len() {
            assert_ne!(
                encodings[i], encodings[j],
                "Encoded paths should be unique: {} vs {}",
                encodings[i], encodings[j]
            );
        }
    }

    // Should contain friendly name from the path
    assert!(
        encoded1.starts_with("notes-"),
        "Should start with 'notes-': {}",
        encoded1
    );
    assert!(
        encoded4.starts_with("_notes-"),
        "Should start with '_notes-': {}",
        encoded4
    );
    assert!(
        encoded5.starts_with("_notes-"),
        "Should start with '_notes-': {}",
        encoded5
    );

    // Should have hash suffix (6 hex chars)
    for encoding in &encodings {
        assert!(
            encoding.contains('-'),
            "Should contain dash separator: {}",
            encoding
        );
        let parts: Vec<&str> = encoding.split('-').collect();
        assert_eq!(
            parts.len(),
            2,
            "Should have exactly one dash separator: {}",
            encoding
        );
        assert_eq!(
            parts[1].len(),
            6,
            "Hash part should be 6 characters: {}",
            encoding
        );
        assert!(
            parts[1].chars().all(|c| c.is_ascii_hexdigit()),
            "Hash part should be hex digits: {}",
            encoding
        );
    }
}
