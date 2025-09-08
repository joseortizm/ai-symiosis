use crate::config::{generate_config_template, get_config_path};
use std::fs;

#[tauri::command]
pub fn get_config_content() -> Result<String, String> {
    let config_path = get_config_path();

    match fs::read_to_string(&config_path) {
        Ok(content) => Ok(content),
        Err(_) => {
            let template = generate_config_template();
            Ok(template)
        }
    }
}

#[tauri::command]
pub fn config_exists(app_state: tauri::State<crate::core::state::AppState>) -> bool {
    !app_state
        .was_first_run()
        .load(std::sync::atomic::Ordering::Relaxed)
}
