use std::fs;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn list_notes(query: Option<&str>) -> Result<Vec<String>, String> {
    if let Some(query) = query {
        if query.is_empty() {
            return Ok(Vec::new());
        }
        const NOTES_DIR: &str = "/Users/dathin/Documents/_notes";
        match fs::read_dir(NOTES_DIR) {
            Ok(entries) => {
                let files = entries
                    .filter_map(|entry| {
                        entry.ok().and_then(|e| {
                            e.path()
                                .file_name()
                                .and_then(|n| n.to_str().map(String::from))
                        })
                    })
                    .filter(|file_name| {
                        file_name.to_lowercase().contains(&query.to_lowercase())
                    })
                    .collect();
                Ok(files)
            }
            Err(e) => Err(e.to_string()),
        }
    } else {
        Ok(Vec::new())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, list_notes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
