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

        // First pass: search filenames only (much faster)
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
                        // Only include results with decent scores to avoid noise
                        if score > 15 || (query.len() == 1 && score > 5) {
                            scored_results.push(NoteResult {
                                filename: file_name.to_string(),
                                score,
                            });
                        }
                    }
                }
            }
        }

        // Only search content if we have very few filename matches and query is quite specific
        if scored_results.len() < 3 && query.len() > 4 {
            for entry in WalkDir::new(NOTES_DIR).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
                        if file_name.starts_with('.') {
                            continue;
                        }

                        // Skip if we already have this file from filename search
                        if scored_results.iter().any(|r| r.filename == file_name) {
                            continue;
                        }

                        // Search in content but only for files we haven't already matched
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            // Limit content search to first part of file for performance
                            let search_content = if content.len() > 1000 {
                                // Find a safe character boundary near 1000 bytes
                                let mut end = 1000;
                                while end > 0 && !content.is_char_boundary(end) {
                                    end -= 1;
                                }
                                &content[..end]
                            } else {
                                &content
                            };

                            if let Some(match_result) = best_match(query, search_content) {
                                let score = match_result.score();
                                // Lower threshold for content matches since they're less relevant
                                if score > 30 {
                                    scored_results.push(NoteResult {
                                        filename: file_name.to_string(),
                                        score: score / 2, // Reduce content match scores
                                    });
                                }
                            }
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_notes, open_note])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
