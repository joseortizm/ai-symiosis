//! Validation Unit Tests
//!
//! Tests for note name validation and security functions.

use crate::utilities::validation::validate_note_name;

#[test]
fn test_validate_note_name_valid_names() {
    assert!(validate_note_name("note.md").is_ok());
    assert!(validate_note_name("my-note.txt").is_ok());
    assert!(validate_note_name("folder/note.md").is_ok());
    assert!(validate_note_name("deep/folder/structure/note.md").is_ok());
    assert!(validate_note_name("note_with_underscores.md").is_ok());
    assert!(validate_note_name("123-numbers.md").is_ok());
}

#[test]
fn test_validate_note_name_path_traversal_attacks() {
    assert!(validate_note_name("../secret.txt").is_err());
    assert!(validate_note_name("folder/../secret.txt").is_err());
    assert!(validate_note_name("../../etc/passwd").is_err());
    assert!(validate_note_name("folder/../../secret.txt").is_err());
    assert!(validate_note_name("../../../root.txt").is_err());
}

#[test]
fn test_validate_note_name_absolute_paths() {
    assert!(validate_note_name("/etc/passwd").is_err());
    assert!(validate_note_name("/home/user/secret.txt").is_err());
    assert!(validate_note_name("/tmp/malicious.sh").is_err());
    assert!(validate_note_name("C:\\Windows\\System32\\config").is_err());
    assert!(validate_note_name("D:\\secrets\\file.txt").is_err());
}

#[test]
fn test_validate_note_name_backslash_rejection() {
    assert!(validate_note_name("folder\\note.txt").is_err());
    assert!(validate_note_name("deep\\folder\\note.md").is_err());
    assert!(validate_note_name("note\\..\\secret.txt").is_err());
}

#[test]
fn test_validate_note_name_hidden_files() {
    assert!(validate_note_name(".hidden").is_err());
    assert!(validate_note_name(".config").is_err());
    assert!(validate_note_name(".ssh/id_rsa").is_err());
    assert!(validate_note_name("folder/.hidden").is_ok());
}

#[test]
fn test_validate_note_name_empty_and_whitespace() {
    let whitespace_variants = vec![
        "",
        "   ",
        "\t",
        "\n",
        "\r",
        "\r\n",
        "\t\t\t",
        "\n\n\n",
        "  \t  \n  ",
        " \t \n \r ",
    ];

    for variant in whitespace_variants {
        assert!(validate_note_name(variant).is_err());
    }
}

#[test]
fn test_validate_note_name_length_limits() {
    assert!(validate_note_name(&"a".repeat(256)).is_err());
    assert!(validate_note_name(&"a".repeat(255)).is_ok());
    assert!(validate_note_name(&"a".repeat(254)).is_ok());
}

#[test]
fn test_validate_note_name_edge_cases() {
    assert!(validate_note_name("../").is_err());
    assert!(validate_note_name("./").is_err());
    assert!(validate_note_name("note..md").is_ok());
    assert!(validate_note_name("folder/../subfolder/note.md").is_err());
    assert!(validate_note_name("folder/file.txt").is_ok());
    assert!(validate_note_name("a/b/c/file.txt").is_ok());
}

#[test]
fn test_security_critical_functions_integration() {
    assert!(validate_note_name("test-note.md").is_ok());
    assert!(validate_note_name("../../../secret.txt").is_err());

    let error_msg = validate_note_name("../../../secret.txt").unwrap_err();
    assert!(error_msg.to_string().contains("Path traversal not allowed"));
}
