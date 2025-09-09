// Configuration module for Symiosis
// Contains essential configuration structures, validation, and management logic

use crate::core::{AppError, AppResult};
use crate::utilities::paths::{get_config_path, get_default_notes_dir};
use crate::utilities::validation::{
    validate_basic_shortcut_format, validate_font_size, validate_notes_directory,
    validate_shortcut_format,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::Shortcut;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigReloadResult {
    Unchanged,
    NotesDirChanged,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub notes_directory: String,
    #[serde(default = "default_global_shortcut")]
    pub global_shortcut: String,

    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub interface: InterfaceConfig,

    #[serde(default)]
    pub editor: EditorConfig,

    #[serde(default)]
    pub shortcuts: ShortcutsConfig,

    #[serde(default)]
    pub preferences: PreferencesConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InterfaceConfig {
    pub ui_theme: String,
    pub font_family: String,
    pub font_size: u16,
    pub editor_font_family: String,
    pub editor_font_size: u16,
    pub markdown_render_theme: String,
    pub md_render_code_theme: String,
    pub always_on_top: bool,
    #[serde(default = "default_window_decorations")]
    pub window_decorations: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutsConfig {
    pub create_note: String,
    pub rename_note: String,
    pub delete_note: String,
    pub edit_note: String,
    pub save_and_exit: String,
    pub open_external: String,
    pub open_folder: String,
    pub refresh_cache: String,
    pub scroll_up: String,
    pub scroll_down: String,
    pub up: String,
    pub down: String,
    pub navigate_previous: String,
    pub navigate_next: String,
    pub navigate_code_previous: String,
    pub navigate_code_next: String,
    pub copy_current_section: String,
    pub open_settings: String,
    pub version_explorer: String,
    pub recently_deleted: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreferencesConfig {
    #[serde(default = "default_max_results")]
    pub max_search_results: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditorConfig {
    pub mode: String,
    pub theme: String,
    pub word_wrap: bool,
    pub tab_size: u16,
    pub expand_tabs: bool,
    pub show_line_numbers: bool,
}

fn default_max_results() -> usize {
    100
}

fn default_global_shortcut() -> String {
    "Ctrl+Shift+N".to_string()
}

fn default_window_decorations() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            notes_directory: get_default_notes_dir(),
            global_shortcut: default_global_shortcut(),
            general: GeneralConfig::default(),
            interface: InterfaceConfig::default(),
            editor: EditorConfig::default(),
            shortcuts: ShortcutsConfig::default(),
            preferences: PreferencesConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {}
    }
}

impl Default for InterfaceConfig {
    fn default() -> Self {
        Self {
            ui_theme: "gruvbox-dark".to_string(),
            font_family: "Inter, sans-serif".to_string(),
            font_size: 14,
            editor_font_family: "JetBrains Mono, Consolas, monospace".to_string(),
            editor_font_size: 14,
            markdown_render_theme: "dark_dimmed".to_string(),
            md_render_code_theme: "gruvbox-dark-medium".to_string(),
            always_on_top: false,
            window_decorations: default_window_decorations(),
        }
    }
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            create_note: "Ctrl+Enter".to_string(),
            rename_note: "Ctrl+m".to_string(),
            delete_note: "Ctrl+x".to_string(),
            edit_note: "Enter".to_string(),
            save_and_exit: "Ctrl+s".to_string(),
            open_external: "Ctrl+o".to_string(),
            open_folder: "Ctrl+f".to_string(),
            refresh_cache: "Ctrl+r".to_string(),
            scroll_up: "Ctrl+u".to_string(),
            scroll_down: "Ctrl+d".to_string(),
            up: "Ctrl+k".to_string(),
            down: "Ctrl+j".to_string(),
            navigate_previous: "Ctrl+p".to_string(),
            navigate_next: "Ctrl+n".to_string(),
            navigate_code_previous: "Ctrl+h".to_string(),
            navigate_code_next: "Ctrl+l".to_string(),
            copy_current_section: "Ctrl+y".to_string(),
            open_settings: "Meta+,".to_string(),
            version_explorer: "Ctrl+/".to_string(),
            recently_deleted: "Ctrl+.".to_string(),
        }
    }
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            max_search_results: default_max_results(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            mode: "basic".to_string(),
            theme: "gruvbox-dark".to_string(),
            word_wrap: true,
            tab_size: 2,
            expand_tabs: true,
            show_line_numbers: true,
        }
    }
}

/// Get available UI themes by scanning the ui-themes directory
pub fn get_available_ui_themes() -> Vec<String> {
    vec!["gruvbox-dark".to_string(), "one-dark".to_string()]
}

/// Get available markdown render themes
pub fn get_available_markdown_themes() -> Vec<String> {
    vec![
        "light".to_string(),
        "dark".to_string(),
        "dark_dimmed".to_string(),
        "auto".to_string(),
        "modern_dark".to_string(),
        "article".to_string(),
        "gruvbox".to_string(),
        "dark_high_contrast".to_string(),
    ]
}

/// Generate a simple configuration template
pub fn generate_config_template() -> String {
    String::new()
}

pub fn parse_shortcut(shortcut_str: &str) -> Option<Shortcut> {
    shortcut_str.parse().ok()
}

pub fn get_config_notes_dir() -> PathBuf {
    let config = load_config();
    PathBuf::from(&config.notes_directory)
}

pub fn get_config_notes_dir_from_config(config: &AppConfig) -> PathBuf {
    PathBuf::from(&config.notes_directory)
}

pub fn load_config() -> AppConfig {
    let config_path = get_config_path();

    match fs::read_to_string(&config_path) {
        Ok(content) => load_config_from_content(&content),
        Err(_) => {
            let default_config = AppConfig::default();
            if let Err(e) = save_config(&default_config) {
                eprintln!("Failed to create default config file: {}", e);
            }
            default_config
        }
    }
}

pub fn load_config_with_first_run_info() -> (AppConfig, bool) {
    let config_path = get_config_path();
    let was_first_run = !config_path.exists();

    let config = match fs::read_to_string(&config_path) {
        Ok(content) => load_config_from_content(&content),
        Err(_) => {
            let default_config = AppConfig::default();
            if let Err(e) = save_config(&default_config) {
                eprintln!("Failed to create default config file: {}", e);
            }
            default_config
        }
    };

    (config, was_first_run)
}

pub fn load_config_from_content(content: &str) -> AppConfig {
    let toml_value = match toml::from_str::<toml::Value>(content) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to parse config TOML: {}. Using defaults.", e);
            return AppConfig::default();
        }
    };

    let notes_directory = extract_notes_directory(&toml_value);
    let global_shortcut = extract_global_shortcut(&toml_value);
    let general = extract_general_config(&toml_value);
    let interface = extract_interface_config(&toml_value);
    let editor = extract_editor_config(&toml_value);
    let shortcuts = extract_shortcuts_config(&toml_value);
    let preferences = extract_preferences_config(&toml_value);

    AppConfig {
        notes_directory,
        global_shortcut,
        general,
        interface,
        editor,
        shortcuts,
        preferences,
    }
}

fn extract_notes_directory(value: &toml::Value) -> String {
    match value.get("notes_directory").and_then(|v| v.as_str()) {
        Some(dir) => {
            if let Err(e) = validate_notes_directory(dir) {
                eprintln!(
                    "Warning: Invalid notes_directory '{}': {}. Using default.",
                    dir, e
                );
                get_default_notes_dir()
            } else {
                dir.to_string()
            }
        }
        None => get_default_notes_dir(),
    }
}

fn extract_global_shortcut(value: &toml::Value) -> String {
    match value.get("global_shortcut").and_then(|v| v.as_str()) {
        Some(shortcut) => {
            if let Err(e) = validate_shortcut_format(shortcut) {
                eprintln!(
                    "Warning: Invalid global_shortcut '{}': {}. Using default.",
                    shortcut, e
                );
                default_global_shortcut()
            } else {
                shortcut.to_string()
            }
        }
        None => default_global_shortcut(),
    }
}

fn extract_general_config(_value: &toml::Value) -> GeneralConfig {
    GeneralConfig::default()
}

fn extract_interface_config(value: &toml::Value) -> InterfaceConfig {
    let interface_section = value.get("interface");
    let mut config = InterfaceConfig::default();

    if let Some(section) = interface_section {
        if let Some(theme) = section.get("ui_theme").and_then(|v| v.as_str()) {
            let valid_themes = get_available_ui_themes();
            if valid_themes.contains(&theme.to_string()) {
                config.ui_theme = theme.to_string();
            } else {
                eprintln!(
                    "Warning: Invalid ui_theme '{}'. Using default '{}'.",
                    theme, config.ui_theme
                );
            }
        }

        if let Some(font) = section.get("font_family").and_then(|v| v.as_str()) {
            config.font_family = font.to_string();
        }

        if let Some(size) = section.get("font_size").and_then(|v| v.as_integer()) {
            let size = size as u16;
            if validate_font_size(size, "UI font size").is_ok() {
                config.font_size = size;
            } else {
                eprintln!(
                    "Warning: Invalid font_size {}. Using default {}.",
                    size, config.font_size
                );
            }
        }

        if let Some(font) = section.get("editor_font_family").and_then(|v| v.as_str()) {
            config.editor_font_family = font.to_string();
        }

        if let Some(size) = section.get("editor_font_size").and_then(|v| v.as_integer()) {
            let size = size as u16;
            if validate_font_size(size, "Editor font size").is_ok() {
                config.editor_font_size = size;
            } else {
                eprintln!(
                    "Warning: Invalid editor_font_size {}. Using default {}.",
                    size, config.editor_font_size
                );
            }
        }

        if let Some(theme) = section
            .get("markdown_render_theme")
            .and_then(|v| v.as_str())
        {
            let valid_themes = get_available_markdown_themes();
            if valid_themes.contains(&theme.to_string()) {
                config.markdown_render_theme = theme.to_string();
            } else {
                eprintln!(
                    "Warning: Invalid markdown_render_theme '{}'. Using default '{}'.",
                    theme, config.markdown_render_theme
                );
            }
        }

        if let Some(theme) = section.get("md_render_code_theme").and_then(|v| v.as_str()) {
            let valid_themes = [
                "gruvbox-dark-hard",
                "gruvbox-dark-medium",
                "gruvbox-dark-soft",
                "gruvbox-light-hard",
                "gruvbox-light-medium",
                "atom-one-dark",
                "dracula",
                "nord",
                "monokai",
                "github-dark",
                "vs2015",
                "night-owl",
                "tokyo-night-dark",
                "atom-one-light",
                "github",
                "vs",
                "xcode",
                "tokyo-night-light",
                "base16-tomorrow-night",
                "base16-ocean",
                "base16-solarized-dark",
                "base16-solarized-light",
                "base16-monokai",
                "base16-dracula",
            ];
            if valid_themes.contains(&theme) {
                config.md_render_code_theme = theme.to_string();
            } else {
                eprintln!(
                    "Warning: Invalid md_render_code_theme '{}'. Using default '{}'.",
                    theme, config.md_render_code_theme
                );
            }
        }

        if let Some(always_top) = section.get("always_on_top").and_then(|v| v.as_bool()) {
            config.always_on_top = always_top;
        }

        if let Some(decorations) = section.get("window_decorations").and_then(|v| v.as_bool()) {
            config.window_decorations = decorations;
        }
    }

    config
}

fn extract_editor_config(value: &toml::Value) -> EditorConfig {
    let editor_section = value.get("editor");
    let mut config = EditorConfig::default();

    if let Some(section) = editor_section {
        if let Some(mode) = section.get("mode").and_then(|v| v.as_str()) {
            let valid_modes = ["basic", "vim", "emacs"];
            if valid_modes.contains(&mode) {
                config.mode = mode.to_string();
            } else {
                eprintln!(
                    "Warning: Invalid editor mode '{}'. Using default '{}'.",
                    mode, config.mode
                );
            }
        }

        if let Some(theme) = section.get("theme").and_then(|v| v.as_str()) {
            let valid_themes = [
                "abcdef",
                "abyss",
                "android-studio",
                "andromeda",
                "basic-dark",
                "basic-light",
                "forest",
                "github-dark",
                "github-light",
                "gruvbox-dark",
                "gruvbox-light",
                "material-dark",
                "material-light",
                "monokai",
                "nord",
                "palenight",
                "solarized-dark",
                "solarized-light",
                "tokyo-night-day",
                "tokyo-night-storm",
                "volcano",
                "vscode-dark",
                "vscode-light",
            ];
            if valid_themes.contains(&theme) {
                config.theme = theme.to_string();
            } else {
                eprintln!(
                    "Warning: Invalid editor theme '{}'. Using default '{}'.",
                    theme, config.theme
                );
            }
        }

        if let Some(wrap) = section.get("word_wrap").and_then(|v| v.as_bool()) {
            config.word_wrap = wrap;
        }

        if let Some(size) = section.get("tab_size").and_then(|v| v.as_integer()) {
            let size = size as u16;
            if size > 0 && size <= 16 {
                config.tab_size = size;
            } else {
                eprintln!(
                    "Warning: Invalid tab_size {}. Using default {}.",
                    size, config.tab_size
                );
            }
        }

        if let Some(expand) = section.get("expand_tabs").and_then(|v| v.as_bool()) {
            config.expand_tabs = expand;
        }

        if let Some(show_numbers) = section.get("show_line_numbers").and_then(|v| v.as_bool()) {
            config.show_line_numbers = show_numbers;
        }
    }

    config
}

fn extract_shortcuts_config(value: &toml::Value) -> ShortcutsConfig {
    let shortcuts_section = value.get("shortcuts");
    let mut config = ShortcutsConfig::default();

    if let Some(section) = shortcuts_section {
        macro_rules! extract_shortcut {
            ($field:ident, $key:literal) => {
                if let Some(shortcut) = section.get($key).and_then(|v| v.as_str()) {
                    if validate_basic_shortcut_format(shortcut).is_ok() {
                        config.$field = shortcut.to_string();
                    } else {
                        eprintln!(
                            "Warning: Invalid shortcut '{}' for {}. Using default '{}'.",
                            shortcut, $key, config.$field
                        );
                    }
                }
            };
        }

        extract_shortcut!(create_note, "create_note");
        extract_shortcut!(rename_note, "rename_note");
        extract_shortcut!(delete_note, "delete_note");
        extract_shortcut!(edit_note, "edit_note");
        extract_shortcut!(save_and_exit, "save_and_exit");
        extract_shortcut!(open_external, "open_external");
        extract_shortcut!(open_folder, "open_folder");
        extract_shortcut!(refresh_cache, "refresh_cache");
        extract_shortcut!(scroll_up, "scroll_up");
        extract_shortcut!(scroll_down, "scroll_down");
        extract_shortcut!(up, "up");
        extract_shortcut!(down, "down");
        extract_shortcut!(navigate_previous, "navigate_previous");
        extract_shortcut!(navigate_next, "navigate_next");
        extract_shortcut!(open_settings, "open_settings");
        extract_shortcut!(version_explorer, "version_explorer");
        extract_shortcut!(recently_deleted, "recently_deleted");
    }

    config
}

fn extract_preferences_config(value: &toml::Value) -> PreferencesConfig {
    let preferences_section = value.get("preferences");
    let mut config = PreferencesConfig::default();

    if let Some(section) = preferences_section {
        if let Some(max_results) = section
            .get("max_search_results")
            .and_then(|v| v.as_integer())
        {
            let max_results = max_results as usize;
            if max_results > 0 && max_results <= 10000 {
                config.max_search_results = max_results;
            } else {
                eprintln!(
                    "Warning: Invalid max_search_results {}. Using default {}.",
                    max_results, config.max_search_results
                );
            }
        }
    }

    config
}

pub fn save_config(config: &AppConfig) -> AppResult<()> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let toml_content = toml::to_string_pretty(config)
        .map_err(|e| AppError::ConfigSave(format!("Failed to serialize config: {}", e)))?;

    fs::write(&config_path, toml_content)?;

    println!("Config saved to: {}", config_path.display());
    Ok(())
}

pub fn reload_config(
    app_config: &std::sync::RwLock<AppConfig>,
    app_handle: Option<AppHandle>,
) -> Result<ConfigReloadResult, String> {
    let new_config = load_config();

    let result = {
        let old_config = app_config
            .read()
            .map_err(|_| "Failed to acquire read lock on config".to_string())?;

        if get_config_notes_dir_from_config(&old_config)
            != get_config_notes_dir_from_config(&new_config)
        {
            ConfigReloadResult::NotesDirChanged
        } else {
            ConfigReloadResult::Unchanged
        }
    };

    let mut config = app_config
        .write()
        .map_err(|_| "Failed to acquire write lock on config".to_string())?;
    *config = new_config.clone();
    drop(config);

    if let Some(app) = app_handle {
        if let Err(e) = app.emit("config-updated", &new_config) {
            eprintln!("Failed to emit config-updated event: {}", e);
        }
    }
    Ok(result)
}
