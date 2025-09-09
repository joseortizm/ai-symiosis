use crate::logging::log;
use std::path::PathBuf;

pub fn encode_path_for_backup(notes_dir: &std::path::Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Get the last component of the path for a friendly name
    let friendly_name = notes_dir
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("notes"))
        .to_string_lossy()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    // Hash the full absolute path to guarantee uniqueness
    let mut hasher = DefaultHasher::new();
    notes_dir.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();

    // Create short hash (6 hex chars should be enough for uniqueness)
    let short_hash = format!("{:06x}", hash & 0xFFFFFF);

    // Combine friendly name with hash: "notes-3f8c9a"
    format!("{}-{}", friendly_name, short_hash)
}

pub fn get_data_dir() -> Option<PathBuf> {
    get_data_dir_impl()
}

fn get_data_dir_impl() -> Option<PathBuf> {
    if let Some(home_dir) = home::home_dir() {
        #[cfg(target_os = "macos")]
        return Some(home_dir.join("Library").join("Application Support"));

        #[cfg(target_os = "windows")]
        return std::env::var("APPDATA").ok().map(PathBuf::from);

        #[cfg(target_os = "linux")]
        return Some(home_dir.join(".local").join("share"));

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return Some(home_dir.join(".local").join("share"));
    }
    None
}

pub fn get_default_notes_dir() -> String {
    if let Some(home_dir) = home::home_dir() {
        home_dir
            .join("Documents")
            .join("Notes")
            .to_string_lossy()
            .to_string()
    } else {
        "./notes".to_string()
    }
}

pub fn get_config_path() -> PathBuf {
    #[cfg(test)]
    {
        if std::env::var("SYMIOSIS_TEST_MODE_ENABLED").is_ok() {
            if let Ok(test_config_path) = std::env::var("SYMIOSIS_TEST_CONFIG_PATH") {
                if test_config_path.contains("/tmp/")
                    || test_config_path.contains("tmp")
                    || test_config_path.contains("/T/")
                {
                    return PathBuf::from(test_config_path);
                } else {
                    log(
                        "PATH_SAFETY",
                        &format!(
                            "SAFETY ERROR: Test config path '{}' is not in temp directory!",
                            test_config_path
                        ),
                        None,
                    );
                }
            }
        }
    }

    if let Some(home_dir) = home::home_dir() {
        home_dir.join(".symiosis").join("config.toml")
    } else {
        PathBuf::from(".symiosis/config.toml")
    }
}
