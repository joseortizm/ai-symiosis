use std::fs;
use sublime_fuzzy::best_match;
use walkdir::WalkDir;

#[derive(serde::Serialize)]
struct NoteResult {
    filename: String,
    score: isize,
}

#[tauri::command]
fn list_notes(query: &str) -> Result<Vec<String>, String> {
    const NOTES_DIR: &str = "/Users/dathin/Documents/_notes";
    let mut results = Vec::new();

    if query.is_empty() {
        // Return all files when no query
        for entry in WalkDir::new(NOTES_DIR).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
                    // Skip system files
                    if !file_name.starts_with('.') {
                        results.push(file_name.to_string());
                    }
                }
            }
        }
        results.sort();
    } else {
        let mut scored_results = Vec::new();

        // Only search filenames for better performance
        for entry in WalkDir::new(NOTES_DIR).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
                    // Skip system files
                    if file_name.starts_with('.') {
                        continue;
                    }

                    // Search in filename with minimum score threshold
                    if let Some(match_result) = best_match(query, file_name) {
                        let score = match_result.score();
                        // More lenient scoring for better results
                        if score > 10 || (query.len() <= 2 && score > 3) {
                            scored_results.push(NoteResult {
                                filename: file_name.to_string(),
                                score,
                            });
                        }
                    }
                }
            }
        }

        // Sort by score (descending) then by filename
        scored_results.sort_by(|a, b| {
            b.score.cmp(&a.score).then_with(|| a.filename.cmp(&b.filename))
        });

        // Limit results to prevent UI freezing
        scored_results.truncate(50);

        // Extract just the filenames
        results = scored_results.into_iter().map(|r| r.filename).collect();
    }

    Ok(results)
}

#[tauri::command]
fn open_note(note_name: &str) -> Result<(), String> {
    const NOTES_DIR: &str = "/Users/dathin/Documents/_notes";
    let note_path = std::path::Path::new(NOTES_DIR).join(note_name);

    open::that(&note_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_note_content(note_name: &str) -> Result<String, String> {
    const NOTES_DIR: &str = "/Users/dathin/Documents/_notes";
    let note_path = std::path::Path::new(NOTES_DIR).join(note_name);

    if !note_path.exists() {
        return Err(format!("File does not exist: {}", note_name));
    }

    fs::read_to_string(&note_path)
        .map_err(|e| format!("Failed to read file '{}': {}", note_name, e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_notes, open_note, get_note_content]) // Fixed: Added get_note_content
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
