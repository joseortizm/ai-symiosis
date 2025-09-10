use crate::core::{AppError, AppResult};
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
    let short_hash = format!("{:06x}", hash & 0xFFFFFF);

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
        #[cfg(debug_assertions)]
        {
            let dev_config_path = home_dir.join(".symiosis-dev").join("config.toml");
            if dev_config_path.exists() {
                return home_dir
                    .join("Documents")
                    .join("Notes-dev")
                    .to_string_lossy()
                    .to_string();
            }
        }

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
                    crate::logging::log(
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

    #[cfg(debug_assertions)]
    {
        if let Some(home_dir) = home::home_dir() {
            let dev_config_path = home_dir.join(".symiosis-dev").join("config.toml");
            if dev_config_path.exists() {
                return dev_config_path;
            }
        }
    }

    if let Some(home_dir) = home::home_dir() {
        home_dir.join(".symiosis").join("config.toml")
    } else {
        PathBuf::from(".symiosis/config.toml")
    }
}

pub fn get_database_path() -> AppResult<PathBuf> {
    let notes_dir = crate::config::get_config_notes_dir();
    get_database_path_for_notes_dir(&notes_dir)
}

pub fn get_database_path_for_notes_dir(notes_dir: &std::path::Path) -> AppResult<PathBuf> {
    let encoded_path = encode_path_for_backup(notes_dir);
    get_data_dir()
        .ok_or_else(|| AppError::ConfigLoad("Failed to get data directory".to_string()))
        .map(|path| {
            path.join("symiosis")
                .join("databases")
                .join(encoded_path)
                .join("notes.sqlite")
        })
}

pub fn get_backup_dir_for_notes_path(notes_dir: &std::path::Path) -> AppResult<PathBuf> {
    let encoded_path = encode_path_for_backup(notes_dir);
    get_data_dir()
        .ok_or_else(|| AppError::ConfigLoad("Failed to get data directory".to_string()))
        .map(|path| path.join("symiosis").join("backups").join(encoded_path))
}

pub fn get_temp_dir() -> AppResult<PathBuf> {
    get_data_dir()
        .ok_or_else(|| AppError::ConfigLoad("Failed to get data directory".to_string()))
        .map(|path| path.join("symiosis").join("temp"))
}
