use std::fs;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn list_notes() -> Result<Vec<String>, String> {
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
                .collect();
            Ok(files)
        }
        Err(e) => Err(e.to_string()),
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
