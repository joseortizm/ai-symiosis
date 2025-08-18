// Centralized logging module for Symiosis
// Provides comprehensive logging for database operations, CRUD, and debugging

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

// Global logging configuration
const LOGGING_ENABLED: bool = true; // TODO: Move to config file later

// Static logger instance
static LOGGER: OnceLock<Mutex<BufWriter<File>>> = OnceLock::new();

fn get_log_path() -> Result<PathBuf, String> {
    crate::database::get_data_dir()
        .ok_or_else(|| "Failed to get data directory".to_string())
        .map(|path| path.join("symiosis").join("symiosis.log"))
}

fn get_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let timestamp = d.as_secs();
            let datetime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
            format!("{:?}", datetime)
        })
        .unwrap_or_else(|_| "UNKNOWN_TIME".to_string())
}

fn init_logger() -> Result<(), String> {
    if !LOGGING_ENABLED {
        return Ok(());
    }

    let log_path = get_log_path()?;

    // Create directory if it doesn't exist
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create log directory: {}", e))?;
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    let writer = BufWriter::new(file);

    LOGGER
        .set(Mutex::new(writer))
        .map_err(|_| "Failed to initialize logger".to_string())?;

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

/// Database-specific logging (backwards compatibility)
pub fn log_db_event(operation: &str, message: &str, details: Option<&str>) {
    log(&format!("DB_{}", operation), message, details);
}

/// CRUD operation logging
pub fn log_crud(operation: &str, entity: &str, details: Option<&str>) {
    log(
        &format!("CRUD_{}", operation),
        &format!("{} operation", entity),
        details,
    );
}

/// File system operation logging
pub fn log_fs(operation: &str, message: &str, details: Option<&str>) {
    log(&format!("FS_{}", operation), message, details);
}

/// Frontend API call logging
pub fn log_api(command: &str, message: &str, details: Option<&str>) {
    log(&format!("API_{}", command), message, details);
}


/// System/lifecycle logging
pub fn log_system(event: &str, message: &str, details: Option<&str>) {
    log(&format!("SYS_{}", event), message, details);
}
