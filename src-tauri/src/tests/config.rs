//! Config Unit Tests
//!
//! Tests config loading, parsing, and validation functionality.
//! These tests access internal/private functions and test the actual production behavior.

use crate::config::{
    get_config_path, get_default_notes_dir, load_config, parse_shortcut, reload_config,
    validate_config, validate_editor_config, validate_notes_directory, validate_preferences_config,
    validate_shortcut_format, AppConfig, EditorConfig,
};

#[test]
fn test_default_config_values() {
    let config = AppConfig::default();

    assert_eq!(config.preferences.max_search_results, 100);
    assert_eq!(config.global_shortcut, "Ctrl+Shift+N");
    assert_eq!(config.editor.mode, "basic");
    assert_eq!(config.interface.markdown_render_theme, "dark_dimmed");
    // notes_directory should be ~/Documents/Notes or ./notes fallback
    assert!(config.notes_directory.contains("Notes") || config.notes_directory == "./notes");
}

#[test]
fn test_get_default_notes_dir() {
    let notes_dir = get_default_notes_dir();
    // Should be either ~/Documents/Notes or ./notes fallback
    assert!(notes_dir.contains("Documents") || notes_dir == "./notes");
    assert!(!notes_dir.is_empty());
}

#[test]
fn test_get_config_path() {
    let config_path = get_config_path();
    // Should be ~/.symiosis/config.toml or .symiosis/config.toml
    let path_str = config_path.to_string_lossy();
    assert!(path_str.contains(".symiosis"));
    assert!(path_str.ends_with("config.toml"));
}

#[test]
fn test_config_toml_serialization_roundtrip() {
    let config = AppConfig::default();
    let toml_str = toml::to_string(&config).expect("Config serialization should work");
    let deserialized: AppConfig =
        toml::from_str(&toml_str).expect("Config deserialization should work");

    assert_eq!(
        config.preferences.max_search_results,
        deserialized.preferences.max_search_results
    );
    assert_eq!(config.notes_directory, deserialized.notes_directory);
    assert_eq!(config.global_shortcut, deserialized.global_shortcut);
    assert_eq!(config.editor.mode, deserialized.editor.mode);
    assert_eq!(
        config.interface.markdown_render_theme,
        deserialized.interface.markdown_render_theme
    );
}

#[test]
fn test_config_toml_serde_defaults() {
    // Test that missing fields use serde defaults
    let minimal_toml = r#"
notes_directory = "/tmp/test"
"#;

    let config: AppConfig = toml::from_str(minimal_toml).expect("Should deserialize with defaults");

    // Specified field
    assert_eq!(config.notes_directory, "/tmp/test");
    // Missing fields should use defaults
    assert_eq!(config.preferences.max_search_results, 100);
    assert_eq!(config.global_shortcut, "Ctrl+Shift+N");
    assert_eq!(config.editor.mode, "basic");
    assert_eq!(config.interface.markdown_render_theme, "dark_dimmed");
}

#[test]
fn test_config_toml_partial_config() {
    let partial_toml = r#"
notes_directory = "/custom/notes"
global_shortcut = "Alt+Space"

[preferences]
max_search_results = 50
"#;

    let config: AppConfig =
        toml::from_str(partial_toml).expect("Should deserialize partial config");

    // Specified fields
    assert_eq!(config.notes_directory, "/custom/notes");
    assert_eq!(config.preferences.max_search_results, 50);
    assert_eq!(config.global_shortcut, "Alt+Space");
    // Missing fields should use defaults
    assert_eq!(config.editor.mode, "basic");
    assert_eq!(config.interface.markdown_render_theme, "dark_dimmed");
}

#[test]
fn test_shortcut_parsing() {
    // Valid shortcuts
    assert!(parse_shortcut("Ctrl+Shift+N").is_some());
    assert!(parse_shortcut("Alt+Space").is_some());
    assert!(parse_shortcut("Cmd+F1").is_some());

    // Invalid shortcuts
    assert!(parse_shortcut("invalid").is_none());
    assert!(parse_shortcut("").is_none());
    assert!(parse_shortcut("Not+A+Real+Shortcut").is_none());
}

#[test]
fn test_load_config_behavior() {
    // load_config() reads from fixed path ~/.symiosis/config.toml
    // If file doesn't exist or parsing fails, it returns defaults
    // We can't easily test file reading without affecting the actual config,
    // but we can test that it doesn't crash and returns reasonable values
    let config = load_config();

    // Should have reasonable values (either from file or defaults)
    assert!(config.preferences.max_search_results > 0);
    assert!(!config.global_shortcut.is_empty());
    assert!(!config.editor.mode.is_empty());
    assert!(!config.interface.markdown_render_theme.is_empty());
    assert!(!config.notes_directory.is_empty());
}

#[test]
fn test_toml_parsing_handles_malformed_input() {
    // Test that malformed TOML fails to parse (as expected by load_config fallback logic)
    let invalid_toml = r#"
notes_directory = "/path  # Missing closing quote
max_search_results = "not_a_number"
invalid_syntax =
"#;

    let result = toml::from_str::<AppConfig>(invalid_toml);
    assert!(result.is_err(), "Malformed TOML should fail to parse");

    // This demonstrates how load_config() behaves: falls back to defaults on parse error
}

#[test]
fn test_save_config_content_validates_toml() {
    // save_config_content validates TOML before saving
    // We test the validation logic without actually saving files

    // Valid TOML should parse successfully
    let valid_toml = r#"
notes_directory = "/valid/path"

[preferences]
max_search_results = 150

[editor]
mode = "vim"
"#;
    let parse_result = toml::from_str::<AppConfig>(valid_toml);
    assert!(parse_result.is_ok(), "Valid TOML should parse successfully");

    // Invalid TOML should fail (this is what save_config_content checks)
    let invalid_toml = r#"
notes_directory = "/path"

[preferences]
max_search_results = "not_a_number"
"#;
    let invalid_result = toml::from_str::<AppConfig>(invalid_toml);
    assert!(
        invalid_result.is_err(),
        "Invalid TOML should fail validation"
    );
}
