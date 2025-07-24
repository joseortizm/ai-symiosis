use comrak::{markdown_to_html, ComrakOptions};
use dirs;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    notes_directory: String,
    #[serde(default = "default_max_results")]
    max_search_results: usize,
    #[serde(default = "default_fuzzy_threshold")]
    fuzzy_match_threshold: u16,
    #[serde(default)]
    editor_settings: EditorSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct EditorSettings {
    #[serde(default = "default_theme")]
    theme: String,
    #[serde(default = "default_font_size")]
    font_size: u16,
}

fn default_max_results() -> usize {
    100
}
fn default_fuzzy_threshold() -> u16 {
    30
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_font_size() -> u16 {
    14
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            notes_directory: get_default_notes_dir(),
            max_search_results: default_max_results(),
            fuzzy_match_threshold: default_fuzzy_threshold(),
            editor_settings: EditorSettings::default(),
        }
    }
}

fn get_default_notes_dir() -> String {
    if let Some(home_dir) = dirs::home_dir() {
        home_dir
            .join("Documents")
            .join("Notes")
            .to_string_lossy()
            .to_string()
    } else {
        "./notes".to_string()
    }
}

fn get_config_path() -> PathBuf {
    if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".symiosis").join("config.toml")
    } else {
        PathBuf::from(".symiosis/config.toml")
    }
}

fn load_config() -> AppConfig {
    let config_path = get_config_path();

    match fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str::<AppConfig>(&content) {
            Ok(config) => {
                println!("Loaded config from: {}", config_path.display());
                config
            }
            Err(e) => {
                eprintln!("Failed to parse config file: {}. Using defaults.", e);
                AppConfig::default()
            }
        },
        Err(_) => {
            println!(
                "Config file not found, creating default config at: {}",
                config_path.display()
            );
            let default_config = AppConfig::default();

            if let Some(parent) = config_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            if let Ok(toml_content) = toml::to_string(&default_config) {
                let _ = fs::write(&config_path, toml_content);
            }

            default_config
        }
    }
}

static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(load_config);

fn get_notes_dir() -> PathBuf {
    PathBuf::from(&APP_CONFIG.notes_directory)
}

fn get_database_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("./"))
        .join("symiosis")
        .join("notes.sqlite")
}

fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS notes USING fts5(filename, content, modified UNINDEXED);")
}

fn load_all_notes_into_sqlite(conn: &mut Connection) -> rusqlite::Result<()> {
    let notes_dir = get_notes_dir();

    // Check if directory exists - if not, just return without creating it
    if !notes_dir.exists() {
        return Ok(());
    }

    conn.execute("DELETE FROM notes", [])?;

    let tx = conn.transaction()?;

    for entry in WalkDir::new(&notes_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            let relative = path.strip_prefix(&notes_dir).unwrap_or(path);
            let filename = relative.to_string_lossy().to_string();

            if filename.contains("/.") || filename.starts_with('.') {
                continue;
            }

            let content = fs::read_to_string(path).unwrap_or_default();
            let modified = entry
                .path()
                .metadata()
                .and_then(|m| m.modified())
                .map(|mtime| {
                    mtime
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            tx.execute(
                "INSERT INTO notes (filename, content, modified) VALUES (?1, ?2, ?3)",
                params![filename, content, modified],
            )?;
        }
    }

    tx.commit()
}

#[tauri::command]
fn list_all_notes() -> Result<Vec<String>, String> {
    let db_path = get_database_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT filename FROM notes ORDER BY modified DESC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for r in rows {
        if let Ok(filename) = r {
            results.push(filename);
        }
    }

    Ok(results)
}

#[tauri::command]
fn search_notes(query: &str) -> Result<Vec<String>, String> {
    let db_path = get_database_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // If query is empty, return all notes ordered by most recent
    if query.trim().is_empty() {
        let mut stmt = conn
            .prepare("SELECT filename FROM notes ORDER BY modified DESC LIMIT ?")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([APP_CONFIG.max_search_results], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        for r in rows {
            if let Ok(filename) = r {
                results.push(filename);
            }
        }
        return Ok(results);
    }

    // For non-empty queries, use FTS search with fuzzy matching
    let mut stmt = conn
        .prepare("SELECT filename FROM notes WHERE notes MATCH ? ORDER BY rank LIMIT ?")
        .map_err(|e| e.to_string())?;

    // Create a more flexible search pattern for fuzzy matching
    let pattern = if query.contains(' ') {
        // For multi-word queries, search for each word
        query
            .split_whitespace()
            .map(|word| format!("{}*", word.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" OR ")
    } else {
        // For single words, use prefix matching with wildcard
        format!("{}*", query.replace('"', ""))
    };

    let rows = stmt
        .query_map(params![pattern, APP_CONFIG.max_search_results], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for r in rows {
        if let Ok(filename) = r {
            results.push(filename);
        }
    }

    Ok(results)
}

#[tauri::command]
fn get_note_content(note_name: &str) -> Result<String, String> {
    let note_path = get_notes_dir().join(note_name);
    if !note_path.exists() {
        return Err(format!("Note not found: {}", note_name));
    }
    let content = fs::read_to_string(&note_path).map_err(|e| e.to_string())?;
    Ok(render_note(note_name, &content))
}

#[tauri::command]
fn get_note_raw_content(note_name: &str) -> Result<String, String> {
    let note_path = get_notes_dir().join(note_name);
    if !note_path.exists() {
        return Err(format!("Note not found: {}", note_name));
    }
    let content = fs::read_to_string(&note_path).map_err(|e| e.to_string())?;
    Ok(content) // Return raw content without rendering
}

fn render_note(filename: &str, content: &str) -> String {
    if filename.ends_with(".md") || filename.ends_with(".markdown") {
        markdown_to_html(content, &ComrakOptions::default())
    } else {
        format!("<pre>{}</pre>", html_escape::encode_text(content))
    }
}

#[tauri::command]
fn save_note(note_name: &str, content: &str) -> Result<(), String> {
    let note_path = get_notes_dir().join(note_name);

    if let Some(parent) = note_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&note_path, content).map_err(|e| e.to_string())?;

    let conn = Connection::open(get_database_path()).map_err(|e| e.to_string())?;
    let modified = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Use INSERT OR REPLACE to handle both insert and update cases
    conn.execute(
        "INSERT OR REPLACE INTO notes (filename, content, modified) VALUES (?1, ?2, ?3)",
        params![note_name, content, modified],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn refresh_cache() -> Result<(), String> {
    let mut conn = Connection::open(get_database_path()).map_err(|e| e.to_string())?;
    init_db(&conn).map_err(|e| e.to_string())?;
    load_all_notes_into_sqlite(&mut conn).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn initialize_notes() {
    let db_path = get_database_path();
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut conn = Connection::open(&db_path).expect("Failed to open DB");
    init_db(&conn).expect("Failed to init DB");
    load_all_notes_into_sqlite(&mut conn).expect("Failed to load notes");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    initialize_notes();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            search_notes,
            get_note_content,
            get_note_raw_content,
            save_note,
            refresh_cache,
            list_all_notes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
