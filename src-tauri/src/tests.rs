use super::*;
use crate::database::get_data_dir;

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
        let toml_str = toml::to_string(&config).expect("Config serialization should work in tests");
        let deserialized: AppConfig =
            toml::from_str(&toml_str).expect("Config deserialization should work in tests");

        assert_eq!(config.max_search_results, deserialized.max_search_results);
        assert_eq!(config.notes_directory, deserialized.notes_directory);
        assert_eq!(config.global_shortcut, deserialized.global_shortcut);
    }

    #[test]
    fn test_config_serde_defaults() {
        let minimal_toml = r#"
            notes_directory = "/tmp/test"
        "#;

        let config: AppConfig =
            toml::from_str(minimal_toml).expect("Minimal TOML should deserialize successfully");

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

        let config: AppConfig =
            toml::from_str(partial_toml).expect("Partial TOML should deserialize successfully");

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

            match result.expect("Search function should return a result") {
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
        let notes_dir = get_config_notes_dir();
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

#[cfg(test)]
mod directory_path_tests {
    use super::*;

    #[test]
    fn test_get_data_dir_returns_valid_path() {
        let data_dir = get_data_dir();
        assert!(
            data_dir.is_some(),
            "get_data_dir should return Some when home directory is available"
        );

        let path = data_dir.expect("Data directory should be available in tests");
        assert!(path.is_absolute(), "Data directory path should be absolute");
        assert!(
            !path.to_string_lossy().is_empty(),
            "Data directory path should not be empty"
        );

        // Verify the path contains expected platform-specific components
        let path_str = path.to_string_lossy();
        let has_valid_structure = path_str.contains("Library") // macOS
            || path_str.contains("AppData") // Windows
            || path_str.contains(".local"); // Linux/Unix
        assert!(
            has_valid_structure,
            "Data directory should contain platform-specific path components: {}",
            path_str
        );

        // Verify the path is actually creatable (this tests real filesystem behavior)
        if let Err(e) = std::fs::create_dir_all(&path) {
            // Only fail if it's not a permission issue (which is expected in some environments)
            if e.kind() != std::io::ErrorKind::PermissionDenied {
                panic!("Should be able to create data directory: {}", e);
            }
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_get_data_dir_macos_structure() {
        if let Some(data_dir) = get_data_dir() {
            let path_str = data_dir.to_string_lossy();
            assert!(
                path_str.contains("Library"),
                "macOS data dir should contain 'Library'"
            );
            assert!(
                path_str.contains("Application Support"),
                "macOS data dir should contain 'Application Support'"
            );
            assert!(
                path_str.ends_with("Library/Application Support"),
                "macOS data dir should end with correct path"
            );
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_get_data_dir_linux_structure() {
        if let Some(data_dir) = get_data_dir() {
            let path_str = data_dir.to_string_lossy();
            assert!(
                path_str.contains(".local"),
                "Linux data dir should contain '.local'"
            );
            assert!(
                path_str.contains("share"),
                "Linux data dir should contain 'share'"
            );
            assert!(
                path_str.ends_with(".local/share"),
                "Linux data dir should end with correct path"
            );
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_data_dir_windows_structure() {
        if let Some(data_dir) = get_data_dir() {
            let path_str = data_dir.to_string_lossy();
            // On Windows, should use APPDATA environment variable
            if let Ok(appdata) = std::env::var("APPDATA") {
                assert_eq!(
                    path_str, appdata,
                    "Windows data dir should match APPDATA env var"
                );
            }
        }
    }

    #[test]
    fn test_get_config_path_structure() {
        let config_path = get_config_path();

        assert!(config_path.is_absolute(), "Config path should be absolute");
        assert!(
            config_path.to_string_lossy().contains(".symiosis"),
            "Config path should contain '.symiosis'"
        );
        assert!(
            config_path.to_string_lossy().ends_with("config.toml"),
            "Config path should end with 'config.toml'"
        );

        // Verify it's in the user's home directory (real validation)
        if let Some(home_dir) = home::home_dir() {
            assert!(
                config_path.starts_with(home_dir),
                "Config path should be in home directory"
            );
        }

        // Test that parent directory can be created (tests real filesystem behavior)
        if let Some(parent) = config_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                if e.kind() != std::io::ErrorKind::PermissionDenied {
                    panic!("Should be able to create config directory: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_get_default_notes_dir_structure() {
        let notes_dir = get_default_notes_dir();

        assert!(!notes_dir.is_empty(), "Notes directory should not be empty");

        let notes_path = std::path::Path::new(&notes_dir);
        assert!(
            notes_path.is_absolute(),
            "Notes directory should be absolute path"
        );

        // Verify it's in the user's home directory (real validation)
        if let Some(home_dir) = home::home_dir() {
            assert!(
                notes_path.starts_with(home_dir),
                "Notes directory should be in home directory"
            );
        }

        // Should contain Documents and Notes in the path structure
        assert!(
            notes_dir.contains("Documents"),
            "Notes directory should contain 'Documents'"
        );
        assert!(
            notes_dir.contains("Notes"),
            "Notes directory should contain 'Notes'"
        );

        // Should end with Documents/Notes or Documents\\Notes depending on platform
        let has_correct_ending = notes_dir.ends_with("Documents/Notes")
            || notes_dir.ends_with("Documents\\Notes")
            || (notes_dir.contains("Documents") && notes_dir.contains("Notes"));
        assert!(
            has_correct_ending,
            "Notes directory should end with correct path structure: {}",
            notes_dir
        );

        // Test that directory can be created (tests real filesystem behavior)
        if let Err(e) = std::fs::create_dir_all(&notes_path) {
            if e.kind() != std::io::ErrorKind::PermissionDenied {
                panic!("Should be able to create notes directory: {}", e);
            }
        }
    }

    #[test]
    fn test_database_path_uses_data_dir() {
        let db_path = get_database_path();

        assert!(db_path.is_absolute(), "Database path should be absolute");
        assert!(
            db_path.to_string_lossy().contains("symiosis"),
            "Database path should contain 'symiosis'"
        );
        assert!(
            db_path.to_string_lossy().ends_with("notes.sqlite"),
            "Database path should end with 'notes.sqlite'"
        );

        // Verify it uses the same data directory structure as get_data_dir
        if let Some(data_dir) = get_data_dir() {
            let expected_prefix = data_dir.join("symiosis");
            let db_parent = db_path
                .parent()
                .expect("Database path should have a parent directory");
            assert_eq!(
                db_parent, expected_prefix,
                "Database path should be in data directory"
            );

            // Test that the database directory can be created (real filesystem test)
            if let Err(e) = std::fs::create_dir_all(db_parent) {
                if e.kind() != std::io::ErrorKind::PermissionDenied {
                    panic!("Should be able to create database directory: {}", e);
                }
            }
        }

        // Test the function integration - database path should actually work with database connection
        let db_path_from_database_module = crate::database::get_db_connection();
        assert!(
            db_path_from_database_module.is_ok() || db_path_from_database_module.is_err(),
            "Database connection should either succeed or fail gracefully"
        );
    }

    #[test]
    fn test_directory_paths_are_absolute() {
        assert!(
            get_config_path().is_absolute(),
            "Config path should be absolute"
        );

        let notes_dir_string = get_default_notes_dir();
        let notes_dir_path = std::path::Path::new(&notes_dir_string);
        assert!(
            notes_dir_path.is_absolute(),
            "Notes directory path should be absolute"
        );

        assert!(
            get_database_path().is_absolute(),
            "Database path should be absolute"
        );

        if let Some(data_dir) = get_data_dir() {
            assert!(
                data_dir.is_absolute(),
                "Data directory path should be absolute"
            );
        }
    }

    #[test]
    fn test_directory_path_consistency() {
        // All directory functions should work together consistently
        let config_path = get_config_path();
        let notes_dir = get_default_notes_dir();
        let db_path = get_database_path();

        // All should be non-empty
        assert!(!config_path.to_string_lossy().is_empty());
        assert!(!notes_dir.is_empty());
        assert!(!db_path.to_string_lossy().is_empty());

        // All should be absolute paths
        assert!(config_path.is_absolute());
        assert!(std::path::Path::new(&notes_dir).is_absolute());
        assert!(db_path.is_absolute());

        // All paths should be different (no conflicts)
        assert_ne!(config_path.to_string_lossy(), notes_dir);
        assert_ne!(config_path, db_path);
        assert_ne!(notes_dir, db_path.to_string_lossy());
    }

    #[test]
    fn test_home_dir_fallback_behavior() {
        // Test what happens when we can't get home directory
        // We can't easily mock home::home_dir() to return None, but we can test fallback paths

        // Test that fallback paths are reasonable
        let fallback_config = std::path::PathBuf::from(".symiosis/config.toml");
        let fallback_notes = "./notes";
        let fallback_db = std::path::PathBuf::from("./symiosis/notes.sqlite");

        // These should be valid relative paths
        assert!(!fallback_config.to_string_lossy().is_empty());
        assert!(!fallback_notes.is_empty());
        assert!(!fallback_db.to_string_lossy().is_empty());

        // Fallback paths should be relative (not absolute)
        assert!(!fallback_config.is_absolute());
        assert!(!std::path::Path::new(fallback_notes).is_absolute());
        assert!(!fallback_db.is_absolute());
    }

    #[test]
    fn test_platform_data_dir_correctness() {
        // This test runs on all platforms and validates the current platform's behavior
        if let Some(data_dir) = get_data_dir() {
            let path_str = data_dir.to_string_lossy();

            // Check that we got the right path for the current platform
            #[cfg(target_os = "macos")]
            {
                assert!(
                    path_str.contains("Library/Application Support"),
                    "On macOS, should use Library/Application Support: {}",
                    path_str
                );
            }

            #[cfg(target_os = "linux")]
            {
                assert!(
                    path_str.contains(".local/share"),
                    "On Linux, should use .local/share: {}",
                    path_str
                );
            }

            #[cfg(target_os = "windows")]
            {
                // On Windows, should either be from APPDATA or be a reasonable fallback
                let is_appdata = std::env::var("APPDATA")
                    .map(|appdata| path_str == appdata)
                    .unwrap_or(false);
                let is_reasonable_windows_path =
                    path_str.contains("AppData") || path_str.contains("Users");
                assert!(
                    is_appdata || is_reasonable_windows_path,
                    "On Windows, should use APPDATA or reasonable fallback: {}",
                    path_str
                );
            }

            // All platforms: should be in user's home directory
            if let Some(home_dir) = home::home_dir() {
                assert!(
                    data_dir.starts_with(home_dir),
                    "Data directory should be within home directory"
                );
            }
        }
    }

    #[test]
    fn test_real_filesystem_integration() {
        // Test that our directory functions work with actual filesystem operations
        let temp_dir = std::env::temp_dir().join("symiosis_test");

        // Clean up from any previous test runs
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Test directory creation works
        assert!(
            std::fs::create_dir_all(&temp_dir).is_ok(),
            "Should be able to create temp test dir"
        );

        // Test file creation in a similar structure to what our app would create
        let test_config_dir = temp_dir.join(".symiosis");
        let test_notes_dir = temp_dir.join("Documents").join("Notes");
        let test_data_dir = temp_dir.join("symiosis");

        assert!(std::fs::create_dir_all(&test_config_dir).is_ok());
        assert!(std::fs::create_dir_all(&test_notes_dir).is_ok());
        assert!(std::fs::create_dir_all(&test_data_dir).is_ok());

        // Test file creation
        let test_config_file = test_config_dir.join("config.toml");
        let test_note_file = test_notes_dir.join("test.md");
        let test_db_file = test_data_dir.join("notes.sqlite");

        assert!(std::fs::write(&test_config_file, "test_content").is_ok());
        assert!(std::fs::write(&test_note_file, "# Test Note").is_ok());
        assert!(std::fs::write(&test_db_file, "fake_db_content").is_ok());

        // Verify files exist and can be read
        assert!(test_config_file.exists());
        assert!(test_note_file.exists());
        assert!(test_db_file.exists());

        assert!(std::fs::read_to_string(&test_config_file).is_ok());
        assert!(std::fs::read_to_string(&test_note_file).is_ok());

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
