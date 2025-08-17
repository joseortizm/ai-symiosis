// Configuration module for Symiosis
// Contains essential configuration structures, validation, and management logic

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::Shortcut;

// ============================================================================
// MAIN CONFIGURATION STRUCTURE
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    // === TOP-LEVEL ESSENTIALS ===
    pub notes_directory: String,
    #[serde(default = "default_global_shortcut")]
    pub global_shortcut: String,

    // === CONFIGURATION SECTIONS ===
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

// ============================================================================
// GENERAL CONFIGURATION
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    // Future extensible core settings
}

// ============================================================================
// INTERFACE CONFIGURATION
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InterfaceConfig {
    pub ui_theme: String, // "gruvbox-dark"|"gruvbox-light"|"one-dark"|"github-light"
    pub font_family: String, // "Inter, sans-serif"
    pub font_size: u16,   // 14
    pub editor_font_family: String, // "JetBrains Mono, Consolas, monospace"
    pub editor_font_size: u16, // 14
    pub markdown_render_theme: String, // "dark_dimmed"|"light"|"dark"|"auto"
    pub default_width: u32, // 1200
    pub default_height: u32, // 800
    pub center_on_startup: bool, // true
    pub remember_size: bool, // true
    pub remember_position: bool, // true
    pub always_on_top: bool, // false
}

// ============================================================================
// UNIFIED SHORTCUTS CONFIGURATION
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutsConfig {
    pub create_note: String,       // "Ctrl+Enter"
    pub rename_note: String,       // "Ctrl+m"
    pub delete_note: String,       // "Ctrl+x"
    pub save_and_exit: String,     // "Ctrl+s"
    pub open_external: String,     // "Ctrl+o"
    pub open_folder: String,       // "Ctrl+f"
    pub refresh_cache: String,     // "Ctrl+r"
    pub scroll_up: String,         // "Ctrl+u"
    pub scroll_down: String,       // "Ctrl+d"
    pub vim_up: String,            // "Ctrl+k"
    pub vim_down: String,          // "Ctrl+j"
    pub navigate_previous: String, // "Ctrl+p"
    pub navigate_next: String,     // "Ctrl+n"
    pub open_settings: String,     // "Meta+,"
}

// ============================================================================
// PREFERENCES CONFIGURATION
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreferencesConfig {
    #[serde(default = "default_max_results")]
    pub max_search_results: usize, // 100
}

// ============================================================================
// EDITOR CONFIGURATION (Text Editing Only)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditorConfig {
    pub mode: String,            // "basic"|"vim"|"emacs"
    pub theme: String,           // "gruvbox-dark"
    pub word_wrap: bool,         // true
    pub tab_size: u16,           // 2
    pub show_line_numbers: bool, // true
}

// ============================================================================
// DEFAULT VALUE FUNCTIONS
// ============================================================================

fn default_max_results() -> usize {
    100
}

fn default_global_shortcut() -> String {
    "Ctrl+Shift+N".to_string()
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

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
        Self {
            // Future extensible core settings
        }
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
            default_width: 1200,
            default_height: 800,
            center_on_startup: true,
            remember_size: true,
            remember_position: true,
            always_on_top: false,
        }
    }
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            create_note: "Ctrl+Enter".to_string(),
            rename_note: "Ctrl+m".to_string(),
            delete_note: "Ctrl+x".to_string(),
            save_and_exit: "Ctrl+s".to_string(),
            open_external: "Ctrl+o".to_string(),
            open_folder: "Ctrl+f".to_string(),
            refresh_cache: "Ctrl+r".to_string(),
            scroll_up: "Ctrl+u".to_string(),
            scroll_down: "Ctrl+d".to_string(),
            vim_up: "Ctrl+k".to_string(),
            vim_down: "Ctrl+j".to_string(),
            navigate_previous: "Ctrl+p".to_string(),
            navigate_next: "Ctrl+n".to_string(),
            open_settings: "Meta+,".to_string(),
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
            show_line_numbers: true,
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

pub fn parse_shortcut(shortcut_str: &str) -> Option<Shortcut> {
    shortcut_str.parse().ok()
}

pub fn get_default_notes_dir() -> String {
    if let Some(home_dir) = home::home_dir() {
        home_dir
            .join("Documents")
            .join("Notes")
            .to_string_lossy()
            .to_string()
    } else {
        "./notes".to_string()
    }
}

pub fn get_config_path() -> PathBuf {
    if let Some(home_dir) = home::home_dir() {
        home_dir.join(".symiosis").join("config.toml")
    } else {
        PathBuf::from(".symiosis/config.toml")
    }
}

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

pub fn validate_config(config: &AppConfig) -> Result<(), String> {
    validate_notes_directory(&config.notes_directory)?;
    validate_shortcut_format(&config.global_shortcut)?;
    validate_general_config(&config.general)?;
    validate_interface_config(&config.interface)?;
    validate_editor_config(&config.editor)?;
    validate_shortcuts_config(&config.shortcuts)?;
    validate_preferences_config(&config.preferences)?;
    Ok(())
}

pub fn validate_general_config(_general: &GeneralConfig) -> Result<(), String> {
    // Future validation for general settings
    Ok(())
}

pub fn validate_interface_config(interface: &InterfaceConfig) -> Result<(), String> {
    let valid_themes = ["gruvbox-dark", "gruvbox-light", "one-dark", "github-light"];
    if !valid_themes.contains(&interface.ui_theme.as_str()) {
        return Err(format!(
            "Invalid UI theme '{}'. Valid themes: {}",
            interface.ui_theme,
            valid_themes.join(", ")
        ));
    }

    validate_font_size(interface.font_size, "UI font size")?;
    validate_font_size(interface.editor_font_size, "Editor font size")?;

    let valid_markdown_render_themes = ["light", "dark", "dark_dimmed", "auto"];
    if !valid_markdown_render_themes.contains(&interface.markdown_render_theme.as_str()) {
        return Err(format!(
            "Invalid markdown render theme '{}'. Valid themes: {}",
            interface.markdown_render_theme,
            valid_markdown_render_themes.join(", ")
        ));
    }

    // Validate window settings
    if interface.default_width < 400 || interface.default_width > 10000 {
        return Err("Window width must be between 400 and 10000 pixels".to_string());
    }
    if interface.default_height < 300 || interface.default_height > 8000 {
        return Err("Window height must be between 300 and 8000 pixels".to_string());
    }

    Ok(())
}

pub fn validate_font_size(size: u16, context: &str) -> Result<(), String> {
    if size < 8 || size > 72 {
        return Err(format!("{} must be between 8 and 72 pixels", context));
    }
    Ok(())
}

pub fn validate_shortcuts_config(shortcuts: &ShortcutsConfig) -> Result<(), String> {
    // Note: These shortcuts are used by frontend JavaScript, not Tauri global shortcuts
    // So we only need basic validation, not Tauri shortcut parsing

    validate_basic_shortcut_format(&shortcuts.create_note)?;
    validate_basic_shortcut_format(&shortcuts.rename_note)?;
    validate_basic_shortcut_format(&shortcuts.delete_note)?;
    validate_basic_shortcut_format(&shortcuts.save_and_exit)?;
    validate_basic_shortcut_format(&shortcuts.open_external)?;
    validate_basic_shortcut_format(&shortcuts.open_folder)?;
    validate_basic_shortcut_format(&shortcuts.refresh_cache)?;
    validate_basic_shortcut_format(&shortcuts.scroll_up)?;
    validate_basic_shortcut_format(&shortcuts.scroll_down)?;
    validate_basic_shortcut_format(&shortcuts.vim_up)?;
    validate_basic_shortcut_format(&shortcuts.vim_down)?;
    validate_basic_shortcut_format(&shortcuts.navigate_previous)?;
    validate_basic_shortcut_format(&shortcuts.navigate_next)?;
    validate_basic_shortcut_format(&shortcuts.open_settings)?;

    Ok(())
}

pub fn validate_editor_config(editor: &EditorConfig) -> Result<(), String> {
    let valid_modes = ["basic", "vim", "emacs"];
    if !valid_modes.contains(&editor.mode.as_str()) {
        return Err(format!(
            "Invalid editor mode '{}'. Valid modes: {}",
            editor.mode,
            valid_modes.join(", ")
        ));
    }

    let valid_themes = ["gruvbox-dark", "gruvbox-light", "one-dark", "github-light"];
    if !valid_themes.contains(&editor.theme.as_str()) {
        return Err(format!(
            "Invalid editor theme '{}'. Valid themes: {}",
            editor.theme,
            valid_themes.join(", ")
        ));
    }

    if editor.tab_size == 0 || editor.tab_size > 16 {
        return Err("Tab size must be between 1 and 16".to_string());
    }

    Ok(())
}

pub fn validate_preferences_config(preferences: &PreferencesConfig) -> Result<(), String> {
    if preferences.max_search_results == 0 {
        return Err("Max search results must be greater than 0".to_string());
    }
    if preferences.max_search_results > 10000 {
        return Err("Max search results too large (max: 10000)".to_string());
    }
    Ok(())
}

pub fn validate_shortcut_format(shortcut: &str) -> Result<(), String> {
    // This is for global shortcuts that need to be parsed by Tauri
    if shortcut.trim().is_empty() {
        return Err("Shortcut cannot be empty".to_string());
    }

    // Pre-validate shortcut format before calling parse_shortcut
    if shortcut.contains("++") || shortcut.starts_with('+') || shortcut.ends_with('+') {
        return Err("Invalid shortcut format".to_string());
    }

    // Test that it can be parsed by Tauri's global shortcut system
    match parse_shortcut(shortcut) {
        Some(_) => Ok(()),
        None => Err(format!("Invalid global shortcut format: '{}'", shortcut)),
    }
}

pub fn validate_basic_shortcut_format(shortcut: &str) -> Result<(), String> {
    // This is for frontend shortcuts that are only used by JavaScript
    // We just need basic validation since they're not parsed by Tauri
    if shortcut.trim().is_empty() {
        return Err("Shortcut cannot be empty".to_string());
    }

    // Basic format checks
    if shortcut.contains("++") || shortcut.starts_with('+') || shortcut.ends_with('+') {
        return Err("Invalid shortcut format".to_string());
    }

    Ok(())
}

pub fn validate_notes_directory(dir: &str) -> Result<(), String> {
    if dir.trim().is_empty() {
        return Err("Notes directory cannot be empty".to_string());
    }

    let path = std::path::Path::new(dir);

    // Reject system directories for security
    let dangerous_paths = [
        "/etc",
        "/root",
        "/sys",
        "/proc",
        "/dev",
        "C:\\Windows",
        "C:\\System32",
        "/System",
        "/Library/System",
    ];

    for dangerous in &dangerous_paths {
        if dir.starts_with(dangerous) {
            return Err(format!("Cannot use system directory: {}", dir));
        }
    }

    // Warn about non-absolute paths but allow them
    if !path.is_absolute() {
        eprintln!("Warning: Using relative notes directory: {}", dir);
    }

    Ok(())
}

// ============================================================================
// CONFIG LOADING AND SAVING
// ============================================================================

pub fn load_config() -> AppConfig {
    let config_path = get_config_path();

    match fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str::<AppConfig>(&content) {
            Ok(config) => match validate_config(&config) {
                Ok(()) => config,
                Err(e) => {
                    eprintln!("Invalid config file: {}. Using defaults.", e);
                    AppConfig::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to parse config file: {}. Using defaults.", e);
                AppConfig::default()
            }
        },
        Err(_) => {
            let default_config = AppConfig::default();
            if let Err(e) = save_config(&default_config) {
                eprintln!("Failed to create default config file: {}", e);
            }
            default_config
        }
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create config directory: {}", e));
        }
    }

    let toml_content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, toml_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    println!("Config saved to: {}", config_path.display());
    Ok(())
}

pub fn reload_config(
    app_config: &std::sync::RwLock<AppConfig>,
    app_handle: Option<AppHandle>,
) -> Result<(), String> {
    let new_config = load_config();

    let mut config = app_config
        .write()
        .map_err(|_| "Failed to acquire write lock on config".to_string())?;
    *config = new_config.clone();
    drop(config);

    // Emit config update event to frontend
    if let Some(app) = app_handle {
        if let Err(e) = app.emit("config-updated", &new_config) {
            eprintln!("Failed to emit config-updated event: {}", e);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn save_config_content(content: &str) -> Result<(), String> {
    let config_path = get_config_path();

    // Parse and validate the content before saving
    let config: AppConfig =
        toml::from_str(content).map_err(|e| format!("Failed to parse TOML: {}", e))?;

    validate_config(&config).map_err(|e| format!("Configuration validation failed: {}", e))?;

    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Err(format!("Failed to create config directory: {}", e));
        }
    }

    // Write the content to file
    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    println!("Config content saved to: {}", config_path.display());
    Ok(())
}

// ============================================================================
// TAURI COMMANDS FOR CONFIG ACCESS
// ============================================================================

#[tauri::command]
pub fn get_general_config(app_config: tauri::State<std::sync::RwLock<AppConfig>>) -> GeneralConfig {
    let config = app_config.read().unwrap_or_else(|e| e.into_inner());
    config.general.clone()
}

#[tauri::command]
pub fn get_interface_config(
    app_config: tauri::State<std::sync::RwLock<AppConfig>>,
) -> InterfaceConfig {
    let config = app_config.read().unwrap_or_else(|e| e.into_inner());
    config.interface.clone()
}

#[tauri::command]
pub fn get_editor_config(app_config: tauri::State<std::sync::RwLock<AppConfig>>) -> EditorConfig {
    let config = app_config.read().unwrap_or_else(|e| e.into_inner());
    config.editor.clone()
}

#[tauri::command]
pub fn get_shortcuts_config(
    app_config: tauri::State<std::sync::RwLock<AppConfig>>,
) -> ShortcutsConfig {
    let config = app_config.read().unwrap_or_else(|e| e.into_inner());
    config.shortcuts.clone()
}

#[tauri::command]
pub fn get_preferences_config(
    app_config: tauri::State<std::sync::RwLock<AppConfig>>,
) -> PreferencesConfig {
    let config = app_config.read().unwrap_or_else(|e| e.into_inner());
    config.preferences.clone()
}
