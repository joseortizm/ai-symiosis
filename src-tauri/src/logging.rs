// Centralized logging module for Symiosis
// Provides comprehensive logging for database operations, CRUD, and debugging

use crate::core::{AppError, AppResult};
use crate::utilities::strings::get_timestamp;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const LOGGING_ENABLED: bool = true;

// Static logger instance
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

    // Create directory if it doesn't exist
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

/// Main logging function - logs to symiosis.log
pub fn log(operation: &str, message: &str, details: Option<&str>) {
    if !LOGGING_ENABLED {
        return;
    }

    // Initialize logger if not already done
    if LOGGER.get().is_none() {
        let _ = init_logger();
    }

    if let Some(logger) = LOGGER.get() {
        if let Ok(mut writer) = logger.lock() {
            let timestamp = get_timestamp();
            let log_line = if let Some(details) = details {
                format!("[{}] {}: {} | {}\n", timestamp, operation, message, details)
            } else {
                format!("[{}] {}: {}\n", timestamp, operation, message)
            };

            let _ = writer.write_all(log_line.as_bytes());
            let _ = writer.flush();
        }
    }
}
