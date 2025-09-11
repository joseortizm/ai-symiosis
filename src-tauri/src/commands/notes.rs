use crate::core::AppResult;
use std::sync::Arc;
use std::time::Duration;

/// Helper function to wrap file operations with programmatic operation flag
pub fn with_programmatic_flag<T, F>(
    app_state: &crate::core::state::AppState,
    operation: F,
) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T>,
{
    app_state
        .programmatic_operation_in_progress()
        .store(true, std::sync::atomic::Ordering::Relaxed);

    let result = operation();

    // Spawn background thread to clear flag after delay - NON-BLOCKING
    let prog_flag = Arc::clone(&app_state.programmatic_operation_in_progress);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5)); // Long enough for watcher to process
        prog_flag.store(false, std::sync::atomic::Ordering::Relaxed);
    });

    result
}

// Re-export all note-related commands from their respective modules
// This maintains backward compatibility while organizing the code better
pub use super::note_crud::*;
pub use super::note_external::*;
pub use super::note_search::*;
pub use super::note_versions::*;
