// Configuration module for Symiosis
// Contains essential configuration structures, validation, and management logic

use crate::core::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
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
    pub ui_theme: String,
    pub font_family: String,
    pub font_size: u16,
    pub editor_font_family: String,
    pub editor_font_size: u16,
    pub markdown_render_theme: String,
    pub md_render_code_theme: String,
    pub always_on_top: bool,
}

// ============================================================================
// UNIFIED SHORTCUTS CONFIGURATION
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutsConfig {
    pub create_note: String,
    pub rename_note: String,
    pub delete_note: String,
    pub save_and_exit: String,
    pub open_external: String,
    pub open_folder: String,
    pub refresh_cache: String,
    pub scroll_up: String,
    pub scroll_down: String,
    pub vim_up: String,
    pub vim_down: String,
    pub navigate_previous: String,
    pub navigate_next: String,
    pub navigate_code_previous: String,
    pub navigate_code_next: String,
    pub copy_current_section: String,
    pub open_settings: String,
}

// ============================================================================
// PREFERENCES CONFIGURATION
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreferencesConfig {
    #[serde(default = "default_max_results")]
    pub max_search_results: usize,
}

// ============================================================================
// EDITOR CONFIGURATION (Text Editing Only)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditorConfig {
    pub mode: String,
    pub theme: String,
    pub word_wrap: bool,
    pub tab_size: u16,
    pub expand_tabs: bool,
    pub show_line_numbers: bool,
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
            md_render_code_theme: "gruvbox-dark-medium".to_string(),
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
            navigate_code_previous: "Ctrl+h".to_string(),
            navigate_code_next: "Ctrl+l".to_string(),
            copy_current_section: "Ctrl+y".to_string(),
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
            expand_tabs: true,
            show_line_numbers: true,
        }
    }
}

// ============================================================================
// THEME DETECTION UTILITIES
// ============================================================================

/// Get available UI themes by scanning the ui-themes directory
pub fn get_available_ui_themes() -> Vec<String> {
    // Return all known UI themes - validation moved to frontend
    vec!["gruvbox-dark".to_string(), "one-dark".to_string()]
}

/// Get available markdown render themes - validation moved to frontend
pub fn get_available_markdown_themes() -> Vec<String> {
    // Return all known markdown themes - validation moved to frontend
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

/// Generate a simple configuration template (deprecated - returns empty string)
/// The app now creates a clean default config automatically via AppConfig::default()
pub fn generate_config_template() -> String {
    String::new()
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

pub fn get_config_notes_dir() -> PathBuf {
    let config = crate::APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
    PathBuf::from(&config.notes_directory)
}

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

pub fn validate_config(config: &AppConfig) -> AppResult<()> {
    validate_notes_directory(&config.notes_directory)?;
    validate_shortcut_format(&config.global_shortcut)?;
    validate_general_config(&config.general)?;
    validate_interface_config(&config.interface)?;
    validate_editor_config(&config.editor)?;
    validate_shortcuts_config(&config.shortcuts)?;
    validate_preferences_config(&config.preferences)?;
    Ok(())
}

pub fn validate_general_config(_general: &GeneralConfig) -> AppResult<()> {
    // Future validation for general settings
    Ok(())
}

pub fn validate_interface_config(interface: &InterfaceConfig) -> AppResult<()> {
    let valid_themes = get_available_ui_themes();
    if !valid_themes.contains(&interface.ui_theme) {
        return Err(AppError::ConfigLoad(format!(
            "Invalid UI theme '{}'. Valid themes: {}",
            interface.ui_theme,
            valid_themes.join(", ")
        )));
    }

    validate_font_size(interface.font_size, "UI font size")?;
    validate_font_size(interface.editor_font_size, "Editor font size")?;

    let valid_markdown_render_themes = get_available_markdown_themes();
    if !valid_markdown_render_themes.contains(&interface.markdown_render_theme) {
        return Err(AppError::ConfigLoad(format!(
            "Invalid markdown render theme '{}'. Valid themes: {}",
            interface.markdown_render_theme,
            valid_markdown_render_themes.join(", ")
        )));
    }

    // Modern highlight.js themes (curated selection)
    let valid_md_code_themes = [
        // Gruvbox variants
        "gruvbox-dark-hard",
        "gruvbox-dark-medium",
        "gruvbox-dark-soft",
        "gruvbox-light-hard",
        "gruvbox-light-medium",
        // Popular dark themes
        "atom-one-dark",
        "dracula",
        "nord",
        "monokai",
        "github-dark",
        "vs2015",
        "night-owl",
        "tokyo-night-dark",
        // Popular light themes
        "atom-one-light",
        "github",
        "vs",
        "xcode",
        "tokyo-night-light",
        // Base16 classics
        "base16-tomorrow-night",
        "base16-ocean",
        "base16-solarized-dark",
        "base16-solarized-light",
        "base16-monokai",
        "base16-dracula",
    ];
    if !valid_md_code_themes.contains(&interface.md_render_code_theme.as_str()) {
        return Err(AppError::ConfigLoad(format!(
            "Invalid markdown code theme '{}'. Valid themes: {}",
            interface.md_render_code_theme,
            valid_md_code_themes.join(", ")
        )));
    }

    Ok(())
}

pub fn validate_font_size(size: u16, context: &str) -> AppResult<()> {
    if size < 8 || size > 72 {
        return Err(AppError::ConfigLoad(format!(
            "{} must be between 8 and 72 pixels",
            context
        )));
    }
    Ok(())
}

pub fn validate_shortcuts_config(shortcuts: &ShortcutsConfig) -> AppResult<()> {
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

pub fn validate_editor_config(editor: &EditorConfig) -> AppResult<()> {
    let valid_modes = ["basic", "vim", "emacs"];
    if !valid_modes.contains(&editor.mode.as_str()) {
        return Err(AppError::ConfigLoad(format!(
            "Invalid editor mode '{}'. Valid modes: {}",
            editor.mode,
            valid_modes.join(", ")
        )));
    }

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
    if !valid_themes.contains(&editor.theme.as_str()) {
        return Err(AppError::ConfigLoad(format!(
            "Invalid editor theme '{}'. Valid themes: {}",
            editor.theme,
            valid_themes.join(", ")
        )));
    }

    if editor.tab_size == 0 || editor.tab_size > 16 {
        return Err(AppError::ConfigLoad(
            "Tab size must be between 1 and 16".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_preferences_config(preferences: &PreferencesConfig) -> AppResult<()> {
    if preferences.max_search_results == 0 {
        return Err(AppError::ConfigLoad(
            "Max search results must be greater than 0".to_string(),
        ));
    }
    if preferences.max_search_results > 10000 {
        return Err(AppError::ConfigLoad(
            "Max search results too large (max: 10000)".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_shortcut_format(shortcut: &str) -> AppResult<()> {
    // This is for global shortcuts that need to be parsed by Tauri
    if shortcut.trim().is_empty() {
        return Err(AppError::ConfigLoad("Shortcut cannot be empty".to_string()));
    }

    // Pre-validate shortcut format before calling parse_shortcut
    if shortcut.contains("++") || shortcut.starts_with('+') || shortcut.ends_with('+') {
        return Err(AppError::ConfigLoad("Invalid shortcut format".to_string()));
    }

    // Test that it can be parsed by Tauri's global shortcut system
    match parse_shortcut(shortcut) {
        Some(_) => Ok(()),
        None => Err(AppError::ConfigLoad(format!(
            "Invalid global shortcut format: '{}'",
            shortcut
        ))),
    }
}

pub fn validate_basic_shortcut_format(shortcut: &str) -> AppResult<()> {
    // This is for frontend shortcuts that are only used by JavaScript
    // We just need basic validation since they're not parsed by Tauri
    if shortcut.trim().is_empty() {
        return Err(AppError::ConfigLoad("Shortcut cannot be empty".to_string()));
    }

    // Basic format checks
    if shortcut.contains("++") || shortcut.starts_with('+') || shortcut.ends_with('+') {
        return Err(AppError::ConfigLoad("Invalid shortcut format".to_string()));
    }

    Ok(())
}

pub fn validate_notes_directory(dir: &str) -> AppResult<()> {
    if dir.trim().is_empty() {
        return Err(AppError::ConfigLoad(
            "Notes directory cannot be empty".to_string(),
        ));
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
            return Err(AppError::ConfigLoad(format!(
                "Cannot use system directory: {}",
                dir
            )));
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
        Ok(content) => load_config_from_content(&content),
        Err(_) => {
            crate::WAS_FIRST_RUN.store(true, std::sync::atomic::Ordering::Relaxed);
            let default_config = AppConfig::default();
            if let Err(e) = save_config(&default_config) {
                eprintln!("Failed to create default config file: {}", e);
            }
            default_config
        }
    }
}

pub fn load_config_from_content(content: &str) -> AppConfig {
    // Parse TOML content to flexible Value structure
    let toml_value = match toml::from_str::<toml::Value>(content) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to parse config TOML: {}. Using defaults.", e);
            return AppConfig::default();
        }
    };

    // Extract each field/section independently with fallbacks
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

// ============================================================================
// FIELD EXTRACTION HELPERS
// ============================================================================

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
    // General config is currently empty, just return default
    // When fields are added, implement extraction logic here
    GeneralConfig::default()
}

fn extract_interface_config(value: &toml::Value) -> InterfaceConfig {
    let interface_section = value.get("interface");
    let mut config = InterfaceConfig::default();

    if let Some(section) = interface_section {
        // Extract UI theme
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

        // Extract font family
        if let Some(font) = section.get("font_family").and_then(|v| v.as_str()) {
            config.font_family = font.to_string();
        }

        // Extract font size
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

        // Extract editor font family
        if let Some(font) = section.get("editor_font_family").and_then(|v| v.as_str()) {
            config.editor_font_family = font.to_string();
        }

        // Extract editor font size
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

        // Extract markdown render theme
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

        // Extract markdown code theme
        if let Some(theme) = section.get("md_render_code_theme").and_then(|v| v.as_str()) {
            let valid_themes = [
                // Gruvbox variants
                "gruvbox-dark-hard",
                "gruvbox-dark-medium",
                "gruvbox-dark-soft",
                "gruvbox-light-hard",
                "gruvbox-light-medium",
                // Popular dark themes
                "atom-one-dark",
                "dracula",
                "nord",
                "monokai",
                "github-dark",
                "vs2015",
                "night-owl",
                "tokyo-night-dark",
                // Popular light themes
                "atom-one-light",
                "github",
                "vs",
                "xcode",
                "tokyo-night-light",
                // Base16 classics
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
    }

    config
}

fn extract_editor_config(value: &toml::Value) -> EditorConfig {
    let editor_section = value.get("editor");
    let mut config = EditorConfig::default();

    if let Some(section) = editor_section {
        // Extract editor mode
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

        // Extract editor theme
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

        // Extract word wrap
        if let Some(wrap) = section.get("word_wrap").and_then(|v| v.as_bool()) {
            config.word_wrap = wrap;
        }

        // Extract tab size
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

        // Extract expand tabs
        if let Some(expand) = section.get("expand_tabs").and_then(|v| v.as_bool()) {
            config.expand_tabs = expand;
        }

        // Extract show line numbers
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
        // Helper macro to extract and validate shortcuts
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
        extract_shortcut!(save_and_exit, "save_and_exit");
        extract_shortcut!(open_external, "open_external");
        extract_shortcut!(open_folder, "open_folder");
        extract_shortcut!(refresh_cache, "refresh_cache");
        extract_shortcut!(scroll_up, "scroll_up");
        extract_shortcut!(scroll_down, "scroll_down");
        extract_shortcut!(vim_up, "vim_up");
        extract_shortcut!(vim_down, "vim_down");
        extract_shortcut!(navigate_previous, "navigate_previous");
        extract_shortcut!(navigate_next, "navigate_next");
        extract_shortcut!(open_settings, "open_settings");
    }

    config
}

fn extract_preferences_config(value: &toml::Value) -> PreferencesConfig {
    let preferences_section = value.get("preferences");
    let mut config = PreferencesConfig::default();

    if let Some(section) = preferences_section {
        // Extract max search results
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
