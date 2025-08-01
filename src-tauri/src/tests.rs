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
        assert_eq!(config.global_shortcut, "Ctrl+Shift+N");
        assert!(!config.notes_directory.is_empty());
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize config");
        let deserialized: AppConfig =
            toml::from_str(&toml_str).expect("Failed to deserialize config");

        assert_eq!(config.max_search_results, deserialized.max_search_results);
        assert_eq!(config.notes_directory, deserialized.notes_directory);
        assert_eq!(config.global_shortcut, deserialized.global_shortcut);
    }

    #[test]
    fn test_config_serde_defaults() {
        let minimal_toml = r#"
            notes_directory = "/tmp/test"
        "#;

        let config: AppConfig = toml::from_str(minimal_toml).expect("Failed to deserialize");

        assert_eq!(config.max_search_results, 100);
        assert_eq!(config.global_shortcut, "Ctrl+Shift+N");
        assert_eq!(config.notes_directory, "/tmp/test");
    }

    #[test]
    fn test_config_serde_field_defaults() {
        let partial_toml = r#"
            notes_directory = "/tmp/test"
            max_search_results = 50
            global_shortcut = "Alt+Space"
        "#;

        let config: AppConfig = toml::from_str(partial_toml).expect("Failed to deserialize");

        assert_eq!(config.notes_directory, "/tmp/test");
        assert_eq!(config.max_search_results, 50);
        assert_eq!(config.global_shortcut, "Alt+Space");
    }

    #[test]
    fn test_shortcut_parsing() {
        assert!(parse_shortcut("Ctrl+Shift+N").is_some());
        assert!(parse_shortcut("Alt+Space").is_some());
        assert!(parse_shortcut("Cmd+F1").is_some());
        assert!(parse_shortcut("invalid").is_none());
        assert!(parse_shortcut("").is_none());
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
mod fts_injection_tests {
    use crate::search::search_notes_hybrid;

    #[test]
    fn test_fts_injection_attempts() {
        let injection_attempts = vec![
            "test AND malicious",
            "test OR secret",
            "test NOT public",
            "test NEAR(password, 5)",
            "test) OR (notes MATCH 'secret'",
            "test) AND (filename:private",
            "filename:secret",
            "content:password",
            "* AND NOT filename:public",
            "NOT test",
            "***",
            "*test*",
            "\"test\" OR \"secret\"",
            "\"unclosed quote",
            "(test OR (secret AND password))",
            "test) UNION SELECT * FROM notes WHERE (1=1",
        ];

        for malicious_query in injection_attempts {
            let result = std::panic::catch_unwind(|| search_notes_hybrid(malicious_query, 10));

            assert!(
                result.is_ok(),
                "FTS injection query caused panic: {}",
                malicious_query
            );

            match result.unwrap() {
                Ok(_) => {}
                Err(error_msg) => {
                    assert!(
                        !error_msg.contains("SQL"),
                        "Error message leaked SQL details: {}",
                        error_msg
                    );
                }
            }
        }
    }

    #[test]
    fn test_fts_query_sanitization() {
        let special_chars = vec![
            "test\"quote",
            "test(paren",
            "test*wildcard",
            "test AND operator",
            "test: colon",
        ];

        for query in special_chars {
            let result = search_notes_hybrid(query, 10);

            match result {
                Ok(_) => {}
                Err(error) => {
                    assert!(
                        !error.to_lowercase().contains("syntax error"),
                        "Query resulted in SQL syntax error: {} for input: {}",
                        error,
                        query
                    );
                }
            }
        }
    }

    #[test]
    fn test_fts_parameter_safety() {
        let dangerous_inputs = vec![
            "'; DROP TABLE notes; --",
            "' UNION SELECT * FROM sqlite_master --",
            "\"; DELETE FROM notes; --",
            "test'; INSERT INTO notes VALUES ('hack'); --",
        ];

        for dangerous_input in dangerous_inputs {
            let result = search_notes_hybrid(dangerous_input, 10);

            match result {
                Ok(_) => {}
                Err(error) => {
                    let error_lower = error.to_lowercase();
                    assert!(
                        !error_lower.contains("table") || error_lower.contains("fts"),
                        "Unexpected error type: {}",
                        error
                    );
                }
            }
        }
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
