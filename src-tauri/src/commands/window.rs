use crate::core::AppResult;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    let result = || -> AppResult<()> {
        match app.get_webview_window("main") {
            Some(window) => {
                window.show()?;
                window.set_focus()?;
            }
            None => {
                let _window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
                    .title("Symiosis Notes")
                    .inner_size(1200.0, 800.0)
                    .center()
                    .visible(false)
                    .build()?;
            }
        }
        Ok(())
    }();
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> Result<(), String> {
    let result = || -> AppResult<()> {
        if let Some(window) = app.get_webview_window("main") {
            window.hide()?;
        }
        Ok(())
    }();
    result.map_err(|e| e.to_string())
}
