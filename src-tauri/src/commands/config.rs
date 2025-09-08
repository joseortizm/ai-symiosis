use tauri::{AppHandle, Manager};

use crate::config::{
    generate_config_template, get_available_markdown_themes, get_available_ui_themes,
    get_config_path, load_config_from_content, validate_config, EditorConfig, GeneralConfig,
    InterfaceConfig, PreferencesConfig, ShortcutsConfig,
};
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

#[tauri::command]
pub fn save_config_content(content: &str) -> Result<(), String> {
    let config_path = get_config_path();

    // Parse using the same lenient approach as load_config
    let config = load_config_from_content(content);

    validate_config(&config).map_err(|e| format!("Configuration validation failed: {}", e))?;

    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // Write the content to file
    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    println!("Config content saved to: {}", config_path.display());
    Ok(())
}

#[tauri::command]
pub async fn scan_available_themes(app: AppHandle) -> Result<serde_json::Value, String> {
    let mut ui_themes = Vec::new();
    let mut markdown_themes = Vec::new();

    // Try to scan bundled resources first (production builds)
    if let Some(resource_dir) = app.path().resource_dir().ok() {
        // Scan UI themes from resources
        let ui_themes_path = resource_dir.join("css/ui-themes");
        if ui_themes_path.exists() {
            if let Ok(entries) = fs::read_dir(&ui_themes_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.starts_with("ui-") && filename.ends_with(".css") {
                            let theme_name = filename
                                .strip_prefix("ui-")
                                .and_then(|s| s.strip_suffix(".css"))
                                .unwrap_or(filename);
                            ui_themes.push(theme_name.to_string());
                        }
                    }
                }
            }
        }

        // Scan markdown themes from resources
        let md_themes_path = resource_dir.join("css/md_render_themes");
        if md_themes_path.exists() {
            if let Ok(entries) = fs::read_dir(&md_themes_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".css") {
                            let theme_name = filename.strip_suffix(".css").unwrap_or(filename);
                            markdown_themes.push(theme_name.to_string());
                        }
                    }
                }
            }
        }
    }

    // Fallback to scanning static directory (development builds)
    if ui_themes.is_empty() || markdown_themes.is_empty() {
        let static_ui_path = std::path::Path::new("./static/css/ui-themes");
        let static_md_path = std::path::Path::new("./static/css/md_render_themes");

        if ui_themes.is_empty() && static_ui_path.exists() {
            if let Ok(entries) = fs::read_dir(static_ui_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.starts_with("ui-") && filename.ends_with(".css") {
                            let theme_name = filename
                                .strip_prefix("ui-")
                                .and_then(|s| s.strip_suffix(".css"))
                                .unwrap_or(filename);
                            ui_themes.push(theme_name.to_string());
                        }
                    }
                }
            }
        }

        if markdown_themes.is_empty() && static_md_path.exists() {
            if let Ok(entries) = fs::read_dir(static_md_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".css") {
                            let theme_name = filename.strip_suffix(".css").unwrap_or(filename);
                            markdown_themes.push(theme_name.to_string());
                        }
                    }
                }
            }
        }
    }

    // Final fallback to known themes
    if ui_themes.is_empty() {
        ui_themes = get_available_ui_themes();
    }
    if markdown_themes.is_empty() {
        markdown_themes = get_available_markdown_themes();
    }

    ui_themes.sort();
    markdown_themes.sort();

    Ok(serde_json::json!({
        "ui_themes": ui_themes,
        "markdown_themes": markdown_themes
    }))
}

// ============================================================================
// TAURI COMMANDS FOR CONFIG ACCESS
// ============================================================================

#[tauri::command]
pub fn get_general_config(app_state: tauri::State<crate::core::state::AppState>) -> GeneralConfig {
    let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
    config.general.clone()
}

#[tauri::command]
pub fn get_interface_config(
    app_state: tauri::State<crate::core::state::AppState>,
) -> InterfaceConfig {
    let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
    config.interface.clone()
}

#[tauri::command]
pub fn get_editor_config(app_state: tauri::State<crate::core::state::AppState>) -> EditorConfig {
    let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
    config.editor.clone()
}

#[tauri::command]
pub fn get_shortcuts_config(
    app_state: tauri::State<crate::core::state::AppState>,
) -> ShortcutsConfig {
    let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
    config.shortcuts.clone()
}

#[tauri::command]
pub fn get_preferences_config(
    app_state: tauri::State<crate::core::state::AppState>,
) -> PreferencesConfig {
    let config = app_state.config.read().unwrap_or_else(|e| e.into_inner());
    config.preferences.clone()
}
