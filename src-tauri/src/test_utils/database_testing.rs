//! Database testing utilities
//!
//! This module contains database integrity checking, consistency verification,
//! and other testing utilities that were previously mixed with production code.

use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Result of database integrity check
#[derive(Debug, Clone)]
pub struct IntegrityCheckResult {
    pub is_healthy: bool,
    pub errors: Vec<String>,
    #[allow(dead_code)]
    pub warnings: Vec<String>,
    pub stats: DatabaseStats,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_notes: i64,
    pub total_size_bytes: i64,
    pub largest_file_size: i64,
    pub avg_file_size: f64,
    #[allow(dead_code)]
    pub files_with_issues: i64,
}

/// Comprehensive database integrity check
pub fn check_database_integrity(conn: &Connection) -> Result<IntegrityCheckResult, String> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Run SQLite's built-in integrity check
    let sqlite_check = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to run SQLite integrity check: {}", e))?;

    if sqlite_check != "ok" {
        errors.push(format!("SQLite integrity check failed: {}", sqlite_check));
    }

    // Check FTS5 table structure
    let fts_check = verify_fts_structure(conn)?;
    if let Some(error) = fts_check {
        errors.push(error);
    }

    // Gather database statistics
    let stats = gather_database_stats(conn)?;

    // Check for data anomalies
    let anomaly_warnings = detect_data_anomalies(conn, &stats)?;
    warnings.extend(anomaly_warnings);

    // Check for performance issues
    let perf_warnings = detect_performance_issues(conn, &stats)?;
    warnings.extend(perf_warnings);

    Ok(IntegrityCheckResult {
        is_healthy: errors.is_empty(),
        errors,
        warnings,
        stats,
    })
}

/// Verify FTS5 table structure is correct
fn verify_fts_structure(conn: &Connection) -> Result<Option<String>, String> {
    // Check if notes table exists
    let table_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='notes'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check table existence: {}", e))?;

    if table_count == 0 {
        return Ok(Some("Notes table does not exist".to_string()));
    }

    // Check table schema
    let table_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='notes'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get table schema: {}", e))?;

    if !table_sql.to_uppercase().contains("FTS5") {
        return Ok(Some("Notes table is not an FTS5 virtual table".to_string()));
    }

    // Verify expected columns
    let expected_columns = ["filename", "content", "modified"];
    for column in &expected_columns {
        if !table_sql.to_lowercase().contains(&column.to_lowercase()) {
            return Ok(Some(format!("Missing expected column: {}", column)));
        }
    }

    Ok(None)
}

/// Gather comprehensive database statistics
fn gather_database_stats(conn: &Connection) -> Result<DatabaseStats, String> {
    // Total number of notes
    let total_notes: i64 = conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .map_err(|e| format!("Failed to count notes: {}", e))?;

    // Total size and file size statistics
    let size_stats: (i64, i64, f64) = conn
        .query_row(
            "SELECT SUM(LENGTH(content)), MAX(LENGTH(content)), AVG(LENGTH(content)) FROM notes",
            [],
            |row| {
                Ok((
                    row.get(0).unwrap_or(0),
                    row.get(1).unwrap_or(0),
                    row.get(2).unwrap_or(0.0),
                ))
            },
        )
        .map_err(|e| format!("Failed to get size statistics: {}", e))?;

    // Count files with potential issues
    let files_with_issues = count_problematic_files(conn)?;

    Ok(DatabaseStats {
        total_notes,
        total_size_bytes: size_stats.0,
        largest_file_size: size_stats.1,
        avg_file_size: size_stats.2,
        files_with_issues,
    })
}

/// Count files with potential data issues
fn count_problematic_files(conn: &Connection) -> Result<i64, String> {
    let mut count = 0i64;

    // Files with empty content
    let empty_files: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE LENGTH(TRIM(content)) = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count empty files: {}", e))?;
    count += empty_files;

    // Files with null bytes
    let null_byte_files: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE content LIKE '%' || CHAR(0) || '%'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count null byte files: {}", e))?;
    count += null_byte_files;

    // Files that are suspiciously large (>10MB)
    let large_files: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE LENGTH(content) > ?1",
            params![10 * 1024 * 1024],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count large files: {}", e))?;
    count += large_files;

    Ok(count)
}

/// Detect data anomalies that might indicate corruption
fn detect_data_anomalies(conn: &Connection, stats: &DatabaseStats) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();

    // Check for unusual file size distribution
    if stats.largest_file_size > 1024 * 1024 * 100 {
        // 100MB
        warnings.push(format!(
            "Very large file detected: {} bytes",
            stats.largest_file_size
        ));
    }

    if stats.total_notes > 0 && stats.avg_file_size < 10.0 {
        warnings.push("Average file size suspiciously small".to_string());
    }

    // Check for files with problematic content
    let empty_content_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE LENGTH(TRIM(content)) = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count empty content: {}", e))?;

    if empty_content_count > 0 {
        warnings.push(format!(
            "Files with empty content detected: {}",
            empty_content_count
        ));
    }

    // Check for files with invalid modification timestamps
    let timestamp_issues: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE modified <= 0 OR modified > ?1",
            params![SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check timestamps: {}", e))?;

    if timestamp_issues > 0 {
        warnings.push(format!(
            "Files with invalid timestamps: {}",
            timestamp_issues
        ));
    }

    Ok(warnings)
}

/// Detect potential performance issues
fn detect_performance_issues(
    conn: &Connection,
    stats: &DatabaseStats,
) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();

    // Check if database is getting large
    if stats.total_notes > 10000 {
        warnings.push(format!(
            "Large number of notes ({}): consider optimization",
            stats.total_notes
        ));
    }

    if stats.total_size_bytes > 1024 * 1024 * 1024 {
        // 1GB
        warnings.push(format!(
            "Large database size ({} bytes): consider archiving",
            stats.total_size_bytes
        ));
    }

    // Test FTS search performance
    let search_start = std::time::Instant::now();
    match conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE notes MATCH 'test'",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(_) => {
            let search_duration = search_start.elapsed();
            if search_duration.as_millis() > 1000 {
                warnings.push(format!(
                    "FTS search is slow ({} ms): consider optimization",
                    search_duration.as_millis()
                ));
            }
        }
        Err(e) => {
            warnings.push(format!("FTS5 search failed: {}", e));
        }
    }

    Ok(warnings)
}

/// Quick health check - returns true if database appears healthy
pub fn quick_health_check(conn: &Connection) -> bool {
    // Run basic checks that should always pass
    let basic_checks = vec![
        // Table exists
        (
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='notes'",
            1i64,
        ),
        // Can query the table
        ("SELECT COUNT(*) FROM notes LIMIT 1", -1i64), // Any result is fine
    ];

    for (query, expected_min) in basic_checks {
        match conn.query_row(query, [], |row| row.get::<_, i64>(0)) {
            Ok(result) => {
                if expected_min >= 0 && result < expected_min {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }

    // Test FTS5 search
    if conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE notes MATCH 'test'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .is_err()
    {
        return false;
    }

    true
}

/// Compare filesystem and database states to detect sync issues
pub fn verify_sync_consistency(
    conn: &Connection,
    filesystem_files: &HashMap<String, (String, i64)>, // filename -> (content, modified_time)
) -> Result<Vec<String>, String> {
    let mut inconsistencies = Vec::new();

    // Get database files
    let mut database_files = HashMap::new();
    let mut stmt = conn
        .prepare("SELECT filename, content, modified FROM notes")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| format!("Failed to query database: {}", e))?;

    for row in rows {
        let (filename, content, modified) =
            row.map_err(|e| format!("Failed to read row: {}", e))?;
        database_files.insert(filename, (content, modified));
    }

    // Check for files in database but not filesystem
    for filename in database_files.keys() {
        if !filesystem_files.contains_key(filename) {
            inconsistencies.push(format!(
                "File in database but not filesystem: {}",
                filename
            ));
        }
    }

    // Check for files in filesystem but not database
    for filename in filesystem_files.keys() {
        if !database_files.contains_key(filename) {
            inconsistencies.push(format!(
                "File in filesystem but not database: {}",
                filename
            ));
        }
    }

    // Check for content mismatches
    for (filename, (fs_content, fs_modified)) in filesystem_files {
        if let Some((db_content, db_modified)) = database_files.get(filename) {
            if fs_content != db_content {
                inconsistencies.push(format!("Content mismatch for file: {}", filename));
            }
            if fs_modified != db_modified {
                inconsistencies.push(format!(
                    "Timestamp mismatch for file {}: filesystem={}, database={}",
                    filename, fs_modified, db_modified
                ));
            }
        }
    }

    Ok(inconsistencies)
}