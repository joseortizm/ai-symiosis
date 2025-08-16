//! Config Unit Tests
//!
//! Tests config loading, parsing, and validation functionality.
//! These tests access internal/private functions and test the actual production behavior.

use crate::config::{
    get_config_path, get_default_notes_dir, load_config, parse_shortcut, reload_config,
    validate_config, validate_editor_config, validate_max_search_results, validate_notes_directory,
    validate_shortcut_format, AppConfig, EditorConfig,
};

#[test]
fn test_default_config_values() {
    let config = AppConfig::default();

    assert_eq!(config.max_search_results, 100);
    assert_eq!(config.global_shortcut, "Ctrl+Shift+N");
    assert_eq!(config.editor.mode, "basic");
    assert_eq!(config.editor.markdown_theme, "dark_dimmed");
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

    assert_eq!(config.max_search_results, deserialized.max_search_results);
    assert_eq!(config.notes_directory, deserialized.notes_directory);
    assert_eq!(config.global_shortcut, deserialized.global_shortcut);
    assert_eq!(config.editor.mode, deserialized.editor.mode);
    assert_eq!(
        config.editor.markdown_theme,
        deserialized.editor.markdown_theme
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
    assert_eq!(config.max_search_results, 100);
    assert_eq!(config.global_shortcut, "Ctrl+Shift+N");
    assert_eq!(config.editor.mode, "basic");
    assert_eq!(config.editor.markdown_theme, "dark_dimmed");
}

#[test]
fn test_config_toml_partial_config() {
    let partial_toml = r#"
notes_directory = "/custom/notes"
max_search_results = 50
global_shortcut = "Alt+Space"
"#;

    let config: AppConfig =
        toml::from_str(partial_toml).expect("Should deserialize partial config");

    // Specified fields
    assert_eq!(config.notes_directory, "/custom/notes");
    assert_eq!(config.max_search_results, 50);
    assert_eq!(config.global_shortcut, "Alt+Space");
    // Missing fields should use defaults
    assert_eq!(config.editor.mode, "basic");
    assert_eq!(config.editor.markdown_theme, "dark_dimmed");
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
    assert!(config.max_search_results > 0);
    assert!(!config.global_shortcut.is_empty());
    assert!(!config.editor.mode.is_empty());
    assert!(!config.editor.markdown_theme.is_empty());
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
max_search_results = 150
editor_mode = "vim"
"#;
    let parse_result = toml::from_str::<AppConfig>(valid_toml);
    assert!(parse_result.is_ok(), "Valid TOML should parse successfully");

    // Invalid TOML should fail (this is what save_config_content checks)
    let invalid_toml = r#"
notes_directory = "/path
max_search_results = "not_a_number"
"#;
    let invalid_result = toml::from_str::<AppConfig>(invalid_toml);
    assert!(
        invalid_result.is_err(),
        "Invalid TOML should fail validation"
    );
}

#[test]
fn test_reload_config_succeeds() {
    // reload_config should work without app handle
    use std::sync::RwLock;
    let config = RwLock::new(AppConfig::default());
    let result = reload_config(&config, None);
    assert!(
        result.is_ok(),
        "reload_config should succeed even without app handle"
    );
}

#[test]
fn test_reload_config_without_app_handle() {
    // When called without app handle, should not emit events but should still work
    use std::sync::RwLock;
    let config = RwLock::new(AppConfig::default());
    let result = reload_config(&config, None);
    assert!(
        result.is_ok(),
        "reload_config should work without app handle"
    );

    // Note: No events emitted when app_handle is None
    // Real event testing would require mocking Tauri's AppHandle
}

// TDD Tests for future enhanced validation

#[test]
fn test_config_validation_should_reject_negative_values() {
    // TOML parsing rejects negative values for usize fields automatically
    let config_with_negative = r#"
notes_directory = "/tmp"
max_search_results = -5
global_shortcut = "Ctrl+Shift+N"
editor_mode = "basic"
markdown_theme = "dark_dimmed"
"#;

    let result = toml::from_str::<AppConfig>(config_with_negative);
    assert!(
        result.is_err(),
        "TOML parsing should reject negative usize values"
    );

    // Test zero values should be rejected by validation
    let config_with_zero = AppConfig {
        notes_directory: "/tmp".to_string(),
        max_search_results: 0,
        global_shortcut: "Ctrl+Shift+N".to_string(),
        editor: EditorConfig {
            mode: "basic".to_string(),
            markdown_theme: "dark_dimmed".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Now implemented: validation should reject zero values
    assert!(
        validate_config(&config_with_zero).is_err(),
        "Validation should reject zero max_search_results"
    );
    assert!(
        validate_max_search_results(0).is_err(),
        "Should reject zero max_search_results"
    );
    assert!(
        validate_max_search_results(10001).is_err(),
        "Should reject excessively large max_search_results"
    );
    assert!(
        validate_max_search_results(100).is_ok(),
        "Should accept reasonable max_search_results"
    );
}

#[test]
fn test_config_validation_should_reject_invalid_editor_modes() {
    // Now implemented: Should validate editor_mode against allowed values
    let config = AppConfig {
        notes_directory: "/tmp".to_string(),
        max_search_results: 100,
        global_shortcut: "Ctrl+Shift+N".to_string(),
        editor: EditorConfig {
            mode: "nonexistent_editor".to_string(),
            markdown_theme: "dark_dimmed".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Validation should now reject invalid editor modes
    assert!(
        validate_config(&config).is_err(),
        "Validation should reject invalid editor mode"
    );
    // Test editor mode validation through editor config
    let invalid_editor = EditorConfig {
        mode: "nonexistent_editor".to_string(),
        ..Default::default()
    };
    assert!(
        validate_editor_config(&invalid_editor).is_err(),
        "Should reject invalid editor mode"
    );

    let basic_editor = EditorConfig {
        mode: "basic".to_string(),
        ..Default::default()
    };
    assert!(
        validate_editor_config(&basic_editor).is_ok(),
        "Should accept valid editor mode: basic"
    );

    let vim_editor = EditorConfig {
        mode: "vim".to_string(),
        ..Default::default()
    };
    assert!(
        validate_editor_config(&vim_editor).is_ok(),
        "Should accept valid editor mode: vim"
    );

    let emacs_editor = EditorConfig {
        mode: "emacs".to_string(),
        ..Default::default()
    };
    assert!(
        validate_editor_config(&emacs_editor).is_ok(),
        "Should accept valid editor mode: emacs"
    );
}

#[test]
fn test_config_validation_should_reject_invalid_shortcuts() {
    // Now implemented: Should validate shortcut format before calling parse_shortcut
    let config = AppConfig {
        notes_directory: "/tmp".to_string(),
        max_search_results: 100,
        global_shortcut: "InvalidShortcutFormat".to_string(),
        editor: EditorConfig {
            mode: "basic".to_string(),
            markdown_theme: "dark_dimmed".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Validation should now reject invalid shortcuts
    assert!(
        validate_config(&config).is_err(),
        "Validation should reject invalid shortcut format"
    );
    assert!(
        validate_shortcut_format("InvalidShortcutFormat").is_err(),
        "Should reject invalid shortcut format"
    );
    assert!(
        validate_shortcut_format("").is_err(),
        "Should reject empty shortcut"
    );
    assert!(
        validate_shortcut_format("++").is_err(),
        "Should reject malformed shortcut with double plus"
    );
    assert!(
        validate_shortcut_format("+Ctrl").is_err(),
        "Should reject shortcut starting with plus"
    );

    // Test that valid shortcuts work correctly
    let valid_config = AppConfig {
        notes_directory: "/tmp".to_string(),
        max_search_results: 100,
        global_shortcut: "Ctrl+Shift+N".to_string(),
        editor: EditorConfig {
            mode: "basic".to_string(),
            markdown_theme: "dark_dimmed".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    // This should validate successfully
    assert!(
        validate_config(&valid_config).is_ok(),
        "Valid config should pass validation"
    );
    assert!(
        validate_shortcut_format("Ctrl+Shift+N").is_ok(),
        "Should accept valid shortcut format"
    );
    assert!(
        validate_shortcut_format("Alt+Space").is_ok(),
        "Should accept valid shortcut format"
    );
}

#[test]
fn test_config_validation_should_reject_unsafe_directories() {
    // Now implemented: Should validate notes_directory for security
    let unsafe_directories = vec![
        "/etc/passwd",
        "/root",
        "/sys",
        "/proc",
        "C:\\Windows\\System32",
    ];

    for unsafe_dir in unsafe_directories {
        let config = AppConfig {
            notes_directory: unsafe_dir.to_string(),
            max_search_results: 100,
            global_shortcut: "Ctrl+Shift+N".to_string(),
            editor: EditorConfig {
                mode: "basic".to_string(),
                markdown_theme: "dark_dimmed".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        // Validation should now reject unsafe directories
        assert!(
            validate_config(&config).is_err(),
            "Should reject unsafe directory: {}",
            unsafe_dir
        );
        assert!(
            validate_notes_directory(unsafe_dir).is_err(),
            "Should reject unsafe directory: {}",
            unsafe_dir
        );
    }

    // Test that safe directories are accepted
    let safe_directories = vec![
        "/home/user/Documents/Notes",
        "/Users/user/Documents/Notes",
        "./notes",
        "../notes",
    ];

    for safe_dir in safe_directories {
        assert!(
            validate_notes_directory(safe_dir).is_ok(),
            "Should accept safe directory: {}",
            safe_dir
        );
    }
}

#[test]
fn test_config_validation_should_reject_invalid_markdown_themes() {
    // Test markdown theme validation
    let config = AppConfig {
        notes_directory: "/tmp".to_string(),
        max_search_results: 100,
        global_shortcut: "Ctrl+Shift+N".to_string(),
        editor: EditorConfig {
            mode: "basic".to_string(),
            markdown_theme: "nonexistent_theme".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Validation should reject invalid themes
    assert!(
        validate_config(&config).is_err(),
        "Should reject invalid markdown theme"
    );
    // Test markdown theme validation through editor config
    let invalid_theme_editor = EditorConfig {
        markdown_theme: "nonexistent_theme".to_string(),
        ..Default::default()
    };
    assert!(
        validate_editor_config(&invalid_theme_editor).is_err(),
        "Should reject invalid markdown theme"
    );

    // Test valid themes
    let valid_themes = ["light", "dark", "dark_dimmed", "auto"];
    for theme in valid_themes {
        let valid_theme_editor = EditorConfig {
            markdown_theme: theme.to_string(),
            ..Default::default()
        };
        assert!(
            validate_editor_config(&valid_theme_editor).is_ok(),
            "Should accept valid theme: {}",
            theme
        );
    }
}
