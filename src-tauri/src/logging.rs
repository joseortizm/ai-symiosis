use crate::core::{AppError, AppResult};
use crate::utilities::strings::get_log_timestamp;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const LOGGING_ENABLED: bool = true;

static LOGGER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();

fn get_log_path() -> AppResult<PathBuf> {
    crate::utilities::paths::get_data_dir()
        .ok_or_else(|| AppError::ConfigLoad("Failed to get data directory".to_string()))
        .map(|path| path.join("symiosis").join("symiosis.log"))
}

fn init_logger() -> AppResult<()> {
    if !LOGGING_ENABLED {
        return Ok(());
    }

    let log_path = get_log_path()?;

    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let writer = BufWriter::new(file);

    LOGGER
        .set(Mutex::new(writer))
        .map_err(|_| AppError::ConfigLoad("Failed to initialize logger".to_string()))?;

    log("LOGGER", "Symiosis logger initialized", None);
    Ok(())
}

/// Main logging function - logs to APP_DIR/symiosis.log
pub fn log(operation: &str, message: &str, details: Option<&str>) {
    if !LOGGING_ENABLED {
        return;
    }

    if LOGGER.get().is_none() {
        let _ = init_logger();
    }

    let timestamp = get_log_timestamp();
    let log_line = if let Some(details) = details {
        format!("[{}] {}: {} | {}", timestamp, operation, message, details)
    } else {
        format!("[{}] {}: {}", timestamp, operation, message)
    };

    // Print ERROR messages to stderr in development builds
    #[cfg(debug_assertions)]
    if operation == "ERROR" {
        eprintln!("{}", log_line);
    }

    // Always log to file
    if let Some(logger) = LOGGER.get() {
        if let Ok(mut writer) = logger.lock() {
            let _ = writer.write_all(format!("{}\n", log_line).as_bytes());
            let _ = writer.flush();
        }
    }
}
