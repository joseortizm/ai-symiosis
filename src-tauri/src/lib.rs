use std::fs;
use std::path::{Path, PathBuf};
use sublime_fuzzy::best_match;
use walkdir::WalkDir;

#[derive(serde::Serialize)]
struct NoteResult {
    filename: String,
    score: isize,
}

const NOTES_DIR: &str = "/Users/dathin/Documents/_notes";

// --- Helper Functions ---

fn is_visible_note(path: &Path) -> bool {
    if let Some(path_str) = path.to_str() {
        !path_str.contains("/.") && !path_str.starts_with('.')
    } else {
        false
    }
}

fn collect_all_notes() -> Result<Vec<String>, String> {
    let mut notes = Vec::new();

    for entry in WalkDir::new(NOTES_DIR).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(relative) = entry.path().strip_prefix(NOTES_DIR) {
                if is_visible_note(relative) {
                    if let Some(s) = relative.to_str() {
                        notes.push(s.to_string());
                    }
                }
            }
        }
    }

    notes.sort();
    Ok(notes)
}

fn score_filename(query: &str, filename: &str) -> Option<isize> {
    best_match(query, filename).and_then(|m| {
        let score = m.score();
        if score > 10 || (query.len() <= 2 && score > 3) {
            Some(score * 3) // Boost for filename match
        } else {
            None
        }
    })
}

fn score_content(query: &str, content: &str) -> Option<isize> {
    let content_lower = content.to_lowercase();
    let query_lower = query.to_lowercase();

    if content_lower.contains(&query_lower) {
        let matches = content_lower.matches(&query_lower).count();
        let first_match = content_lower.find(&query_lower).unwrap_or(content.len());
        let position_score = match first_match {
            0..=99 => 20,
            100..=499 => 10,
            _ => 5,
        };
        Some((matches * 15 + position_score) as isize)
    } else if query.len() > 3 {
        let snippet = content.char_indices().take_while(|&(i, _)| i < 2000).map(|(_, c)| c).collect::<String>();
        best_match(query, &snippet).and_then(|m| {
            let score = m.score();
            if score > 25 {
                Some(score / 3) // Lower weight for fuzzy content
            } else {
                None
            }
        })
    } else {
        None
    }
}

fn search_notes(query: &str) -> Result<Vec<String>, String> {
    let mut results = Vec::new();

    for entry in WalkDir::new(NOTES_DIR).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) if !name.starts_with('.') => name,
            _ => continue,
        };

        let mut score = score_filename(query, file_name);

        if score.is_none() {
            if let Ok(content) = fs::read_to_string(path) {
                score = score_content(query, &content);
            }
        }

        if let Some(score) = score {
            if let Ok(relative) = path.strip_prefix(NOTES_DIR) {
                if let Some(s) = relative.to_str() {
                    results.push(NoteResult {
                        filename: s.to_string(),
                        score,
                    });
                }
            }
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.filename.cmp(&b.filename)));
    results.truncate(50);

    Ok(results.into_iter().map(|r| r.filename).collect())
}

// --- Tauri Commands ---

#[tauri::command]
fn list_notes(query: &str) -> Result<Vec<String>, String> {
    if query.trim().is_empty() {
        collect_all_notes()
    } else {
        search_notes(query)
    }
}

#[tauri::command]
fn open_note(note_name: &str) -> Result<(), String> {
    let note_path = Path::new(NOTES_DIR).join(note_name);
    open::that(&note_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_note_content(note_name: &str) -> Result<String, String> {
    let note_path = Path::new(NOTES_DIR).join(note_name);

    if !note_path.exists() {
        return Err(format!("File does not exist: {}", note_name));
    }

    fs::read_to_string(&note_path)
        .map_err(|e| format!("Failed to read file '{}': {}", note_name, e))
}

// --- App Entrypoint ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_notes,
            open_note,
            get_note_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
