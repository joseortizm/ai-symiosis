//! Database Unit Tests
//!
//! Tests for database integration functionality.

use crate::*;

#[test]
fn test_database_path_creation() {
    let db_path = get_database_path();
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
