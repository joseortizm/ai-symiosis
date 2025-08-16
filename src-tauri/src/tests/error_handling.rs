//! Error Handling Unit Tests
//!
//! Tests for error message quality and input validation bounds.

use crate::*;

#[test]
fn test_error_message_quality() {
    let long_name = "a".repeat(256);
    let test_cases = vec![
        ("", "Note name cannot be empty"),
        ("../secret", "Path traversal not allowed"),
        (".hidden", "Note name cannot start with a dot"),
        (long_name.as_str(), "Note name too long"),
        ("file\\path", "Invalid note name"),
    ];

    for (input, expected_error_content) in test_cases {
        let error_msg = validate_note_name(input).unwrap_err();
        assert!(error_msg.contains(expected_error_content));

        if input.len() > 10 {
            assert!(!error_msg.contains(input));
        }
    }
}

#[test]
fn test_function_input_bounds() {
    assert!(validate_note_name(&"a".repeat(10000)).is_err());
    assert!(validate_note_name("note-测试-🦀.md").is_ok());
}
