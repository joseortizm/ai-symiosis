use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

use crate::{
    config::get_config_notes_dir,
    database::with_db,
    services::note_service::{create_versioned_backup, update_note_in_database, BackupType},
};
use std::sync::atomic::{AtomicU32, Ordering};

struct DebouncedWatcher {
    pending_events: Arc<Mutex<HashMap<PathBuf, Instant>>>,
    debounce_duration: Duration,
    cleanup_counter: AtomicU32,
}

impl DebouncedWatcher {
    fn new(debounce_ms: u64) -> Self {
        Self {
            pending_events: Arc::new(Mutex::new(HashMap::new())),
            debounce_duration: Duration::from_millis(debounce_ms),
            cleanup_counter: AtomicU32::new(0),
        }
    }

    fn should_process_event(&self, path: &PathBuf) -> bool {
        let now = Instant::now();
        let mut pending = match self.pending_events.lock() {
            Ok(pending) => pending,
            Err(e) => {
                eprintln!("Watcher lock poisoned, recovering: {}", e);
                e.into_inner()
            }
        };

        if let Some(last_event) = pending.get(path) {
            if now.duration_since(*last_event) < self.debounce_duration {
                return false;
            }
        }

        pending.insert(path.clone(), now);
        true
    }

    fn cleanup_old_events(&self) {
        let now = Instant::now();
        let mut pending = match self.pending_events.lock() {
            Ok(pending) => pending,
            Err(e) => {
                eprintln!("Watcher cleanup lock poisoned, recovering: {}", e);
                e.into_inner()
            }
        };

        let cleanup_threshold = self.debounce_duration * 10;
        pending.retain(|_, &mut last_event| now.duration_since(last_event) < cleanup_threshold);
    }
}

pub fn setup_notes_watcher(
    app_handle: AppHandle,
    app_state: Arc<crate::core::state::AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let notes_dir = get_config_notes_dir();

    std::fs::create_dir_all(&notes_dir)?;

    let debounced_watcher = Arc::new(DebouncedWatcher::new(500));

    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )?;

    watcher.watch(&notes_dir, RecursiveMode::Recursive)?;

    let app_handle_clone = app_handle.clone();
    let debounced_watcher_clone = debounced_watcher.clone();
    let app_state_clone = app_state.clone();
    thread::spawn(move || {
        let _watcher = watcher;

        for event in rx {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    let involves_notes = event.paths.iter().any(|path| {
                        path.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| matches!(ext, "md" | "txt" | "markdown"))
                            .unwrap_or(false)
                    });

                    if involves_notes {
                        if !app_state_clone
                            .programmatic_operation_in_progress()
                            .load(Ordering::Relaxed)
                        {
                            let should_process = event
                                .paths
                                .iter()
                                .any(|path| debounced_watcher_clone.should_process_event(path));

                            if should_process {
                                let app_handle_for_refresh = app_handle_clone.clone();
                                let paths_to_update = event.paths.clone();

                                tauri::async_runtime::spawn(async move {
                                    let notes_dir = get_config_notes_dir();

                                    for path in &paths_to_update {
                                        if let Ok(relative) = path.strip_prefix(&notes_dir) {
                                            let filename = relative.to_string_lossy().to_string();

                                            if filename.contains("/.") || filename.starts_with('.')
                                            {
                                                continue;
                                            }

                                            if path.exists() {
                                                let modified = path
                                                    .metadata()
                                                    .and_then(|m| m.modified())
                                                    .map(|mtime| {
                                                        mtime
                                                            .duration_since(std::time::UNIX_EPOCH)
                                                            .map(|d| d.as_secs() as i64)
                                                            .unwrap_or(0)
                                                    })
                                                    .unwrap_or(0);

                                                if let Ok(content) = std::fs::read_to_string(path) {
                                                    let _ = with_db(|conn| {
                                                        let mut stmt = conn.prepare("SELECT content FROM notes WHERE filename = ?1")?;
                                                        match stmt.query_row(rusqlite::params![filename], |row| {
                                                            Ok(row.get::<_, String>(0)?)
                                                        }) {
                                                            Ok(old_content) => {
                                                                if old_content != content {
                                                                    match create_versioned_backup(path, BackupType::ExternalChange, Some(&old_content)) {
                                                                        Ok(backup_path) => {
                                                                            eprintln!(
                                                                                "Created external change backup: {}",
                                                                                backup_path.display()
                                                                            );
                                                                        }
                                                                        Err(e) => {
                                                                            eprintln!(
                                                                                "Failed to create external change backup for {}: {}",
                                                                                filename, e
                                                                            );
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(_) => {}
                                                        }
                                                        Ok(())
                                                    }).unwrap_or_else(|e| {
                                                        eprintln!("Failed to check for existing content before external change backup: {}", e);
                                                    });

                                                    if let Err(e) = update_note_in_database(
                                                        &filename, &content, modified,
                                                    ) {
                                                        eprintln!(
                                                            "Failed to update note {}: {}",
                                                            filename, e
                                                        );
                                                    }
                                                }
                                            } else {
                                                if let Err(e) = crate::with_db(|conn| {
                                                    conn.execute(
                                                        "DELETE FROM notes WHERE filename = ?1",
                                                        rusqlite::params![filename],
                                                    )
                                                    .map_err(|e| format!("Database error: {}", e))?;
                                                    Ok(())
                                                }) {
                                                    eprintln!(
                                                        "Failed to delete note {}: {}",
                                                        filename, e
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    if let Err(e) =
                                        app_handle_for_refresh.emit("cache-refreshed", ())
                                    {
                                        eprintln!("Failed to emit cache-refreshed event: {}", e);
                                    }
                                });
                            }
                        }
                    }
                }
                _ => {}
            }

            let counter = debounced_watcher_clone
                .cleanup_counter
                .fetch_add(1, Ordering::Relaxed);
            if counter >= 100 {
                debounced_watcher_clone.cleanup_old_events();
                debounced_watcher_clone
                    .cleanup_counter
                    .store(0, Ordering::Relaxed);
            }
        }
    });

    Ok(())
}
