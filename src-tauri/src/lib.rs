use comrak::{markdown_to_html, ComrakOptions};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use walkdir::WalkDir;

// Configuration structure
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

// Default value functions
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

            // Create the directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            // Try to save the default config
            if let Ok(toml_content) = toml::to_string(&default_config) {
                let _ = fs::write(&config_path, toml_content);
            }

            default_config
        }
    }
}

// Global config - loaded once at startup
static APP_CONFIG: LazyLock<Mutex<AppConfig>> = LazyLock::new(|| Mutex::new(load_config()));

// Helper function to get notes directory from config
fn get_notes_dir() -> String {
    APP_CONFIG.lock().unwrap().notes_directory.clone()
}

#[tauri::command]
fn get_note_content(note_name: &str) -> Result<String, String> {
    let cache = NOTES_CACHE.lock().map_err(|e| e.to_string())?;
    if let Some(cached_note) = cache.iter().find(|n| n.filename == note_name) {
        return Ok(render_note(&cached_note.filename, &cached_note.content));
    }
    drop(cache);

    let notes_dir = get_notes_dir();
    let note_path = Path::new(&notes_dir).join(note_name);
    if !note_path.exists() {
        return Err(format!("File does not exist: {note_name}"));
    }

    let content = fs::read_to_string(&note_path)
        .map_err(|e| format!("Failed to read file '{note_name}': {e}"))?;

    Ok(render_note(note_name, &content))
}

fn render_note(filename: &str, content: &str) -> String {
    let is_markdown = filename.ends_with(".md") || filename.ends_with(".markdown");

    if is_markdown {
        markdown_to_html(content, &ComrakOptions::default())
    } else {
        // Wrap plain text in a <pre> tag to preserve formatting
        format!("<pre>{}</pre>", html_escape::encode_text(content))
    }
}

#[derive(serde::Serialize, Clone)]
struct NoteResult {
    filename: String,
    score: u16,
}

#[derive(Clone)]
struct CachedNote {
    filename: String,
    content: String,
}

// Global cache for notes - reuse across searches
// Use Mutex to allow mutation
static NOTES_CACHE: LazyLock<Mutex<Vec<CachedNote>>> =
    LazyLock::new(|| Mutex::new(load_all_notes_into_cache()));

// --- Helper Functions ---

fn is_visible_note(path: &Path) -> bool {
    if let Some(path_str) = path.to_str() {
        !path_str.contains("/.") && !path_str.starts_with('.')
    } else {
        false
    }
}

fn load_all_notes_into_cache() -> Vec<CachedNote> {
    let mut notes = Vec::new();
    let notes_dir = get_notes_dir();

    for entry in WalkDir::new(&notes_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(relative) = entry.path().strip_prefix(&notes_dir) {
                if is_visible_note(relative) {
                    if let Some(filename) = relative.to_str() {
                        // Pre-load content into cache
                        let content = fs::read_to_string(entry.path()).unwrap_or_default();
                        notes.push(CachedNote {
                            filename: filename.to_string(),
                            content,
                        });
                    }
                }
            }
        }
    }

    notes.sort_by(|a, b| a.filename.cmp(&b.filename));
    notes
}

fn collect_all_notes() -> Result<Vec<String>, String> {
    let cache = NOTES_CACHE.lock().map_err(|e| e.to_string())?;
    Ok(cache.iter().map(|note| note.filename.clone()).collect())
}

// Reuse buffers and matcher for better performance
thread_local! {
    static MATCHER: std::cell::RefCell<Matcher> = std::cell::RefCell::new(Matcher::new(Config::DEFAULT));
    static HAYSTACK_BUF: std::cell::RefCell<Vec<char>> = const { std::cell::RefCell::new(Vec::new()) };
    static NEEDLE_BUF: std::cell::RefCell<Vec<char>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn score_filename_improved(query: &str, filename: &str) -> Option<u16> {
    let filename_lower = filename.to_lowercase();
    let query_lower = query.to_lowercase();

    // Check for exact substring match first (highest priority)
    if filename_lower.contains(&query_lower) {
        let position = filename_lower.find(&query_lower).unwrap_or(filename.len());
        let exact_score = match position {
            0 => 2000, // Starts with query - highest score
            _ => 1800, // Contains query - very high score
        };
        return Some(exact_score);
    }

    // Fall back to fuzzy matching, but be much more restrictive
    MATCHER.with(|m| {
        HAYSTACK_BUF.with(|h_buf| {
            NEEDLE_BUF.with(|n_buf| {
                let mut matcher = m.borrow_mut();
                let mut haystack_buf = h_buf.borrow_mut();
                let mut needle_buf = n_buf.borrow_mut();

                haystack_buf.clear();
                needle_buf.clear();

                let haystack = Utf32Str::new(filename, &mut haystack_buf);
                let needle = Utf32Str::new(query, &mut needle_buf);

                let threshold = APP_CONFIG.lock().unwrap().fuzzy_match_threshold;
                matcher.fuzzy_match(haystack, needle).and_then(|score| {
                    // Much more restrictive fuzzy matching for filenames
                    // Only allow very high-quality fuzzy matches to compete
                    if score > threshold && query.len() > 2 {
                        Some((score / 2).min(400)) // Cap fuzzy matches at 400, well below exact content matches
                    } else if query.len() <= 2 && score > 20 {
                        Some((score * 2).min(200))
                    } else {
                        None
                    }
                })
            })
        })
    })
}

fn score_content_improved(query: &str, content: &str) -> Option<u16> {
    let content_lower = content.to_lowercase();
    let query_lower = query.to_lowercase();

    // Fast path: exact substring match (this should beat most fuzzy filename matches)
    if content_lower.contains(&query_lower) {
        let matches = content_lower.matches(&query_lower).count();
        let first_match = content_lower.find(&query_lower).unwrap_or(content.len());
        let position_score = match first_match {
            0..=99 => 40, // Very early in content
            100..=299 => 25,
            300..=999 => 15,
            _ => 8,
        };
        // Much higher boost for exact content matches - should beat fuzzy filename matches
        let base_score = matches * 20 + position_score;
        return Some((base_score * 15).min(65535) as u16); // Aggressive boost
    }

    // Fuzzy match only for longer queries and limit content size
    if query.len() > 3 {
        let snippet = if content.len() > 1500 {
            let mut end = 1500.min(content.len());
            while end > 0 && !content.is_char_boundary(end) {
                end -= 1;
            }
            &content[..end]
        } else {
            content
        };

        MATCHER.with(|m| {
            HAYSTACK_BUF.with(|h_buf| {
                NEEDLE_BUF.with(|n_buf| {
                    let mut matcher = m.borrow_mut();
                    let mut haystack_buf = h_buf.borrow_mut();
                    let mut needle_buf = n_buf.borrow_mut();

                    haystack_buf.clear();
                    needle_buf.clear();

                    let haystack = Utf32Str::new(snippet, &mut haystack_buf);
                    let needle = Utf32Str::new(query, &mut needle_buf);

                    matcher.fuzzy_match(haystack, needle).and_then(|score| {
                        if score > 30 {
                            Some((score / 2).min(300)) // Even fuzzy content matches should be reasonable
                        } else {
                            None
                        }
                    })
                })
            })
        })
    } else {
        None
    }
}

fn search_notes(query: &str) -> Result<Vec<String>, String> {
    if query.trim().is_empty() {
        return collect_all_notes();
    }

    let mut results = Vec::new();

    let cache = NOTES_CACHE.lock().map_err(|e| e.to_string())?;
    for cached_note in cache.iter() {
        // Calculate both filename and content scores using improved functions
        let filename_score = score_filename_improved(query, &cached_note.filename);
        let content_score = score_content_improved(query, &cached_note.content);

        // Use the higher of the two scores
        let final_score = match (filename_score, content_score) {
            (Some(f_score), Some(c_score)) => Some(f_score.max(c_score)),
            (Some(f_score), None) => Some(f_score),
            (None, Some(c_score)) => Some(c_score),
            (None, None) => None,
        };

        if let Some(score) = final_score {
            results.push(NoteResult {
                filename: cached_note.filename.clone(),
                score,
            });
        }
    }

    // Sort by score (descending) then filename (ascending)
    results.sort_unstable_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.filename.cmp(&b.filename))
    });

    // Use max results from config
    let max_results = APP_CONFIG.lock().unwrap().max_search_results;
    results.truncate(max_results);

    Ok(results.into_iter().map(|r| r.filename).collect())
}

// --- Tauri Commands ---

#[tauri::command]
fn list_notes(query: &str) -> Result<Vec<String>, String> {
    search_notes(query)
}

#[tauri::command]
fn open_note(note_name: &str) -> Result<(), String> {
    let notes_dir = get_notes_dir();
    let note_path = Path::new(&notes_dir).join(note_name);
    open::that(&note_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_note_raw_content(note_name: &str) -> Result<String, String> {
    let cache = NOTES_CACHE.lock().map_err(|e| e.to_string())?;
    if let Some(cached_note) = cache.iter().find(|n| n.filename == note_name) {
        return Ok(cached_note.content.clone());
    }
    drop(cache);

    let notes_dir = get_notes_dir();
    let note_path = Path::new(&notes_dir).join(note_name);
    if !note_path.exists() {
        return Err(format!("File does not exist: {note_name}"));
    }

    let content = fs::read_to_string(&note_path)
        .map_err(|e| format!("Failed to read file '{note_name}': {e}"))?;

    Ok(content)
}

#[tauri::command]
fn save_note(note_name: &str, content: &str) -> Result<(), String> {
    let notes_dir = get_notes_dir();
    let note_path = Path::new(&notes_dir).join(note_name);

    // Ensure the parent directory exists
    if let Some(parent) = note_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    // Write the content to the file
    fs::write(&note_path, content)
        .map_err(|e| format!("Failed to write file '{note_name}': {e}"))?;

    // Update the cache with the new content
    let mut cache = NOTES_CACHE.lock().map_err(|e| e.to_string())?;
    if let Some(cached_note) = cache.iter_mut().find(|n| n.filename == note_name) {
        cached_note.content = content.to_string();
    } else {
        // If note doesn't exist in cache, add it
        cache.push(CachedNote {
            filename: note_name.to_string(),
            content: content.to_string(),
        });
        // Re-sort the cache
        cache.sort_by(|a, b| a.filename.cmp(&b.filename));
    }

    Ok(())
}

#[tauri::command]
fn refresh_cache() -> Result<(), String> {
    let mut cache = NOTES_CACHE.lock().map_err(|e| e.to_string())?;
    *cache = load_all_notes_into_cache();
    Ok(())
}

#[tauri::command]
fn get_config() -> Result<AppConfig, String> {
    APP_CONFIG
        .lock()
        .map(|config| config.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_config(new_config: AppConfig) -> Result<(), String> {
    // Update the in-memory config
    {
        let mut config = APP_CONFIG.lock().map_err(|e| e.to_string())?;
        *config = new_config.clone();
    }

    // Save to file
    let config_path = get_config_path();
    let toml_content =
        toml::to_string(&new_config).map_err(|e| format!("Failed to serialize config: {e}"))?;

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {e}"))?;
    }

    fs::write(&config_path, toml_content)
        .map_err(|e| format!("Failed to write config file: {e}"))?;

    // Refresh notes cache if notes directory changed
    refresh_cache()?;

    Ok(())
}

#[tauri::command]
fn reload_config() -> Result<AppConfig, String> {
    let new_config = load_config();
    {
        let mut config = APP_CONFIG.lock().map_err(|e| e.to_string())?;
        *config = new_config.clone();
    }

    // Refresh notes cache since directory might have changed
    refresh_cache()?;

    Ok(new_config)
}

// --- App Entrypoint ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_notes,
            open_note,
            get_note_content,
            refresh_cache,
            get_note_raw_content,
            save_note,
            get_config,
            update_config,
            reload_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
