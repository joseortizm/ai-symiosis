use comrak::{markdown_to_html, ComrakOptions};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use walkdir::WalkDir;

#[tauri::command]
fn get_note_content(note_name: &str) -> Result<String, String> {
    let cache = NOTES_CACHE.lock().map_err(|e| e.to_string())?;
    if let Some(cached_note) = cache.iter().find(|n| n.filename == note_name) {
        return Ok(render_note(&cached_note.filename, &cached_note.content));
    }
    drop(cache);

    let note_path = Path::new(NOTES_DIR).join(note_name);
    if !note_path.exists() {
        return Err(format!("File does not exist: {}", note_name));
    }

    let content = fs::read_to_string(&note_path)
        .map_err(|e| format!("Failed to read file '{}': {}", note_name, e))?;

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

const NOTES_DIR: &str = "/Users/dathin/Documents/_notes";

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

    for entry in WalkDir::new(NOTES_DIR).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(relative) = entry.path().strip_prefix(NOTES_DIR) {
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
    static HAYSTACK_BUF: std::cell::RefCell<Vec<char>> = std::cell::RefCell::new(Vec::new());
    static NEEDLE_BUF: std::cell::RefCell<Vec<char>> = std::cell::RefCell::new(Vec::new());
}

fn score_filename(query: &str, filename: &str) -> Option<u16> {
    MATCHER.with(|m| {
        HAYSTACK_BUF.with(|h_buf| {
            NEEDLE_BUF.with(|n_buf| {
                let mut matcher = m.borrow_mut();
                let mut haystack_buf = h_buf.borrow_mut();
                let mut needle_buf = n_buf.borrow_mut();

                haystack_buf.clear();
                needle_buf.clear();

                // Dereference the RefMut to get &mut Vec<u32>
                let haystack = Utf32Str::new(filename, &mut *haystack_buf);
                let needle = Utf32Str::new(query, &mut *needle_buf);

                matcher.fuzzy_match(haystack, needle).and_then(|score| {
                    if score > 8 || (query.len() <= 2 && score > 2) {
                        Some(score * 4) // Higher boost for filename match
                    } else if score > 0 {
                        Some(score)
                    } else {
                        None
                    }
                })
            })
        })
    })
}

fn score_content(query: &str, content: &str) -> Option<u16> {
    let content_lower = content.to_lowercase();
    let query_lower = query.to_lowercase();

    // Fast path: exact substring match
    if content_lower.contains(&query_lower) {
        let matches = content_lower.matches(&query_lower).count();
        let first_match = content_lower.find(&query_lower).unwrap_or(content.len());
        let position_score = match first_match {
            0..=99 => 25,
            100..=299 => 15,
            300..=999 => 8,
            _ => 3,
        };
        return Some((matches * 12 + position_score).min(65535) as u16);
    }

    // Fuzzy match only for longer queries and limit content size
    if query.len() > 3 {
        let snippet = if content.len() > 1500 {
            // Find a safe character boundary near 1500 bytes
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

                    // Dereference the RefMut to get &mut Vec<u32>
                    let haystack = Utf32Str::new(snippet, &mut *haystack_buf);
                    let needle = Utf32Str::new(query, &mut *needle_buf);

                    matcher.fuzzy_match(haystack, needle).and_then(|score| {
                        if score > 20 {
                            Some(score / 4) // Lower weight for fuzzy content
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
        // Try filename first (higher priority)
        let mut score = score_filename(query, &cached_note.filename);

        // If no filename match, try content
        if score.is_none() {
            score = score_content(query, &cached_note.content);
        }

        if let Some(score) = score {
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

    // Limit results for better UI performance
    results.truncate(100);

    Ok(results.into_iter().map(|r| r.filename).collect())
}

// --- Tauri Commands ---

#[tauri::command]
fn list_notes(query: &str) -> Result<Vec<String>, String> {
    search_notes(query)
}

#[tauri::command]
fn open_note(note_name: &str) -> Result<(), String> {
    let note_path = Path::new(NOTES_DIR).join(note_name);
    open::that(&note_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn refresh_cache() -> Result<(), String> {
    let mut cache = NOTES_CACHE.lock().map_err(|e| e.to_string())?;
    *cache = load_all_notes_into_cache();
    Ok(())
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
            refresh_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
