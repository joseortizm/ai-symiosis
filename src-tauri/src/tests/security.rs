//! Security Unit Tests
//!
//! Tests for security-related functionality and regression tests.

use crate::utilities::validation::validate_note_name;

#[test]
fn test_path_traversal_variations() {
    let rejected_patterns = vec![
        "../",
        "folder/../file",
        "..\\/",
        "../\\/",
        "....\\\\",
        "...\\/...\\/",
    ];

    for pattern in rejected_patterns {
        assert!(validate_note_name(pattern).is_err());
    }
}

#[test]
fn test_filename_injection_attempts() {
    let rejected_names = vec![
        "../etc/passwd",
        "../../../root",
        "C:\\Windows\\System32\\config",
        "/dev/null",
        "/proc/self/mem",
        "\\\\server\\share\\file",
    ];

    for name in rejected_names {
        assert!(validate_note_name(name).is_err());
    }
}
