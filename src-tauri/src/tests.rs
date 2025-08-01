use super::*;

#[cfg(test)]
mod validation_tests {
    use super::*;

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
            "", "   ", "\t", "\n", "\r", "\r\n", "\t\t\t", "\n\n\n", "  \t  \n  ", " \t \n \r ",
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
        assert!(error_msg.contains("Path traversal not allowed"));
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = AppConfig::default();
        
        assert_eq!(config.max_search_results, 100);
        assert_eq!(config.fuzzy_match_threshold, 30);
        
        assert_eq!(config.editor_settings.font_size, 0);
        assert_eq!(config.editor_settings.theme, "");
        
        assert!(!config.notes_directory.is_empty());
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize config");
        let deserialized: AppConfig =
            toml::from_str(&toml_str).expect("Failed to deserialize config");

        assert_eq!(config.max_search_results, deserialized.max_search_results);
        assert_eq!(
            config.fuzzy_match_threshold,
            deserialized.fuzzy_match_threshold
        );
        assert_eq!(
            config.editor_settings.theme,
            deserialized.editor_settings.theme
        );
        assert_eq!(
            config.editor_settings.font_size,
            deserialized.editor_settings.font_size
        );
    }

    #[test]
    fn test_config_serde_defaults() {
        let minimal_toml = r#"
            notes_directory = "/tmp/test"
        "#;
        
        let config: AppConfig = toml::from_str(minimal_toml).expect("Failed to deserialize");
        
        assert_eq!(config.max_search_results, 100);
        assert_eq!(config.fuzzy_match_threshold, 30);
        
        assert_eq!(config.editor_settings.theme, "");
        assert_eq!(config.editor_settings.font_size, 0);
        assert_eq!(config.notes_directory, "/tmp/test");
    }

    #[test]
    fn test_config_serde_field_defaults() {
        let partial_toml = r#"
            notes_directory = "/tmp/test"
            
            [editor_settings]
            font_size = 16
        "#;
        
        let config: AppConfig = toml::from_str(partial_toml).expect("Failed to deserialize");
        
        assert_eq!(config.notes_directory, "/tmp/test");
        assert_eq!(config.editor_settings.theme, "dark");
        assert_eq!(config.editor_settings.font_size, 16);
    }
}

#[cfg(test)]
mod note_rendering_tests {
    use super::*;

    #[test]
    fn test_render_markdown_note() {
        let markdown_content = "# Hello World\n\nThis is **bold** text.";
        let result = render_note("test.md", markdown_content);

        assert!(result.contains("<h1>"));
        assert!(result.contains("Hello World"));
        assert!(result.contains("<strong>"));
        assert!(result.contains("bold"));
    }

    #[test]
    fn test_render_plain_text_note() {
        let text_content = "This is plain text with <script>alert('xss')</script>";
        let result = render_note("test.txt", text_content);

        assert!(result.starts_with("<pre>"));
        assert!(result.ends_with("</pre>"));
        assert!(result.contains("&lt;script&gt;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_render_note_file_extension_detection() {
        let content = "# Test";

        assert!(render_note("test.md", content).contains("<h1>"));
        assert!(render_note("test.markdown", content).contains("<h1>"));

        assert!(render_note("test.txt", content).starts_with("<pre>"));
        assert!(render_note("test.rs", content).starts_with("<pre>"));
        assert!(render_note("no-extension", content).starts_with("<pre>"));
    }
}


#[cfg(test)]
mod database_integration_tests {
    use super::*;

    #[test]
    fn test_database_path_creation() {
        let db_path = get_database_path();
        assert!(db_path.is_absolute());
        assert!(db_path.to_string_lossy().contains("symiosis"));
        assert!(db_path.to_string_lossy().contains("notes.sqlite"));
    }

    #[test]
    fn test_notes_directory_validation() {
        let notes_dir = get_notes_dir();
        assert!(notes_dir.is_absolute());
        assert!(!notes_dir.to_string_lossy().is_empty());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

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
}

#[cfg(test)]
mod security_regression_tests {
    use super::*;

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

}
