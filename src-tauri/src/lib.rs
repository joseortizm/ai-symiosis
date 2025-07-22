use std::fs;
use sublime_fuzzy::best_match;
use walkdir::WalkDir;

#[tauri::command]
fn list_notes(query: &str) -> Result<Vec<String>, String> {
    const NOTES_DIR: &str = "/Users/dathin/Documents/_notes";
    let mut results = Vec::new();

    if query.is_empty() {
        for entry in WalkDir::new(NOTES_DIR).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
                    results.push(file_name.to_string());
                }
            }
        }
    } else {
        for entry in WalkDir::new(NOTES_DIR).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
                    if best_match(query, file_name).is_some() {
                        results.push(file_name.to_string());
                    } else if let Ok(content) = fs::read_to_string(entry.path()) {
                        if best_match(query, &content).is_some() {
                            results.push(file_name.to_string());
                        }
                    }
                }
            }
        }
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
