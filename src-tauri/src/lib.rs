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
        let query_lower = query.to_lowercase();

        for entry in WalkDir::new(NOTES_DIR).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
                    // Skip system files
                    if file_name.starts_with('.') {
                        continue;
                    }

                    let mut best_score = 0;
                    let mut match_found = false;

                    // First, try filename fuzzy matching (highest priority)
                    if let Some(match_result) = best_match(query, file_name) {
                        let score = match_result.score();
                        if score > 10 || (query.len() <= 2 && score > 3) {
                            best_score = score * 3; // Boost filename matches
                            match_found = true;
                        }
                    }

                    // If no good filename match, search content
                    if !match_found {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            let content_lower = content.to_lowercase();

                            // Simple substring search for exact matches (very fast)
                            if content_lower.contains(&query_lower) {
                                // Calculate a simple score based on number of matches and position
                                let matches = content_lower.matches(&query_lower).count();
                                let first_match_pos = content_lower.find(&query_lower).unwrap_or(content.len());

                                // Score: more matches = higher score, earlier position = higher score
                                let position_score = if first_match_pos < 100 { 20 } else if first_match_pos < 500 { 10 } else { 5 };
                                best_score = ((matches * 15) + position_score) as isize;
                                match_found = true;
                            }
                            // Fallback to fuzzy search on content for partial matches (only if query is substantial)
                            else if query.len() > 3 {
                                // Only search first 2000 chars for performance
                                let search_content = if content.len() > 2000 {
                                    let mut end = 2000;
                                    while end > 0 && !content.is_char_boundary(end) {
                                        end -= 1;
                                    }
                                    &content[..end]
                                } else {
                                    &content
                                };

                                if let Some(match_result) = best_match(query, search_content) {
                                    let score = match_result.score();
                                    if score > 25 {
                                        best_score = score / 3; // Reduce fuzzy content match scores
                                        match_found = true;
                                    }
                                }
                            }
                        }
                    }

                    if match_found {
                        scored_results.push(NoteResult {
                            filename: file_name.to_string(),
                            score: best_score,
                        });
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
