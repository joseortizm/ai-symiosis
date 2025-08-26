use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::thread;
use tauri::{AppHandle, Emitter};

use crate::{get_config_notes_dir, refresh_cache, PROGRAMMATIC_OPERATION_IN_PROGRESS};
use std::sync::atomic::Ordering;

pub fn setup_notes_watcher(app_handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let notes_dir = get_config_notes_dir();

    // Create a channel to receive the events
    let (tx, rx) = mpsc::channel();

    // Create a watcher object, delivering raw events
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(&notes_dir, RecursiveMode::Recursive)?;

    // Spawn a thread to handle file system events
    let app_handle_clone = app_handle.clone();
    thread::spawn(move || {
        // Keep the watcher alive
        let _watcher = watcher;

        for event in rx {
            // Filter for relevant file events (create, write, remove, rename)
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    // Check if the event involves markdown/text files
                    let involves_notes = event.paths.iter().any(|path| {
                        path.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| matches!(ext, "md" | "txt" | "markdown"))
                            .unwrap_or(false)
                    });

                    if involves_notes {
                        // Only refresh cache if this is NOT a programmatic operation
                        if !PROGRAMMATIC_OPERATION_IN_PROGRESS.load(Ordering::SeqCst) {
                            // Refresh the cache when note files change
                            // Spawn async task for cache refresh
                            let app_handle_for_refresh = app_handle_clone.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = refresh_cache(app_handle_for_refresh.clone()).await
                                {
                                    eprintln!("Failed to refresh cache after file change: {}", e);
                                } else {
                                    // Emit event to notify frontend of cache refresh
                                    if let Err(e) =
                                        app_handle_for_refresh.emit("cache-refreshed", ())
                                    {
                                        eprintln!("Failed to emit cache-refreshed event: {}", e);
                                    }
                                }
                            });
                        }
                    }
                }
                _ => {} // Ignore other event types
            }
        }
    });

    Ok(())
}
