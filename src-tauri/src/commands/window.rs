use crate::core::AppResult;
use crate::APP_CONFIG;
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
                let config = APP_CONFIG.read().unwrap_or_else(|e| e.into_inner());
                let interface_config = &config.interface;

                let mut window_builder =
                    WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
                        .title("Symiosis Notes")
                        .inner_size(1200.0, 800.0)
                        .center()
                        .visible(false)
                        .decorations(interface_config.window_decorations);

                if interface_config.always_on_top {
                    window_builder = window_builder.always_on_top(true);
                }

                let _window = window_builder.build()?;
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
