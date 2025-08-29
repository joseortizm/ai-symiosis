use crate::commands::notes::save_note_with_content_check;
use crate::config::get_config_notes_dir;
use std::fs;

/// CRITICAL TEST: Editor/Content Synchronization Validation
///
/// This test is absolutely critical for preventing catastrophic data loss.
/// It ensures that the content validation system correctly prevents wrong-file overwrites
/// when the UI state becomes desynchronized from the backend state.
///
/// IMPORTANCE:
/// - Prevents silent data corruption when list selection changes during editing
/// - Catches the exact scenario where user experienced data loss during development
/// - Validates that external file changes are correctly detected
/// - Ensures content comparison consistency between read and save operations
///
/// FAILURE OF THIS TEST indicates a critical vulnerability that MUST be fixed immediately.
/// Any changes to content validation, file reading, or save validation MUST pass this test.
#[test]
fn test_content_synchronization_prevents_data_loss() {
    let notes_dir = get_config_notes_dir();

    // Create two test files with different content
    let file_a = "test_file_a.md";
    let file_b = "test_file_b.md";
    let content_a = "This is file A content";
    let content_b = "This is file B content - completely different";

    // Setup: Create both files
    let path_a = notes_dir.join(file_a);
    let path_b = notes_dir.join(file_b);
    fs::write(&path_a, content_a).unwrap();
    fs::write(&path_b, content_b).unwrap();

    // Simulate opening file A for editing (read directly from filesystem for test)
    let original_content_a = fs::read_to_string(&path_a).unwrap();
    assert_eq!(original_content_a, content_a);

    // User edits file A content (simulating editor state)
    let edited_content_a = "This is file A content - EDITED VERSION";

    // SCENARIO 1: Correct save (file A → file A) should succeed
    let save_result = save_note_with_content_check(file_a, edited_content_a, &original_content_a);
    assert!(save_result.is_ok(), "Correct save should succeed");

    // Verify file A was updated
    let updated_content = fs::read_to_string(&path_a).unwrap();
    assert_eq!(updated_content, edited_content_a);

    // SCENARIO 2: Simulate external file modification (like during development)
    // Modify file B externally (simulating file watcher reordering list)
    let externally_modified_content_b = "File B modified externally";
    fs::write(&path_b, externally_modified_content_b).unwrap();

    // SCENARIO 3: Attempt wrong-target save (file A content → file B)
    // This simulates the data loss scenario: UI thinks it's saving file A but targets file B
    let wrong_save_result = save_note_with_content_check(
        file_b,              // Wrong target (file B)
        edited_content_a,    // Content from file A editor
        &original_content_a, // Original content from when file A was opened
    );

    // CRITICAL: This save MUST fail to prevent data loss
    assert!(
        wrong_save_result.is_err(),
        "Wrong-target save MUST fail to prevent data loss. Got: {:?}",
        wrong_save_result
    );

    // Verify the error message is descriptive
    let error_msg = wrong_save_result.unwrap_err();
    assert!(error_msg.contains("file has been modified since editing began"));

    // CRITICAL: Verify file B was NOT corrupted
    let file_b_content = fs::read_to_string(&path_b).unwrap();
    assert_eq!(
        file_b_content, externally_modified_content_b,
        "File B should remain unchanged after failed save attempt"
    );
    assert_ne!(
        file_b_content, edited_content_a,
        "File B MUST NOT contain content intended for file A"
    );

    // Cleanup
    let _ = fs::remove_file(&path_a);
    let _ = fs::remove_file(&path_b);
}

#[test]
fn test_content_consistency_across_operations() {
    let notes_dir = get_config_notes_dir();

    let file_name = "consistency_test.md";
    let content = "Test content for consistency check";

    // Create file
    let file_path = notes_dir.join(file_name);
    fs::write(&file_path, content).unwrap();

    // Read content (directly from filesystem for test)
    let original_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(original_content, content);

    // Save same content with correct original content should succeed
    let save_result = save_note_with_content_check(file_name, content, &original_content);
    assert!(
        save_result.is_ok(),
        "Save with correct original content should succeed"
    );

    // Cleanup
    let _ = fs::remove_file(&file_path);
}

#[test]
fn test_nonexistent_file_content_handling() {
    let notes_dir = get_config_notes_dir();

    let file_name = "nonexistent.md";

    // Get content of nonexistent file (should be empty string)
    let original_content = String::new();
    assert_eq!(original_content, "");

    // Save to nonexistent file should succeed with empty original content
    let new_content = "New file content";
    let save_result = save_note_with_content_check(file_name, new_content, &original_content);
    assert!(
        save_result.is_ok(),
        "Save to nonexistent file should succeed"
    );

    // Verify file was created
    let file_path = notes_dir.join(file_name);
    assert!(file_path.exists());
    assert_eq!(fs::read_to_string(&file_path).unwrap(), new_content);

    // Cleanup
    let _ = fs::remove_file(&file_path);
}

#[test]
fn test_content_validation_with_external_changes() {
    let notes_dir = get_config_notes_dir();

    let file_name = "external_change_test.md";
    let original_content = "Original content";
    let edited_content = "Edited content";
    let external_content = "Externally modified content";

    // Create file
    let file_path = notes_dir.join(file_name);
    fs::write(&file_path, original_content).unwrap();

    // Get original content (directly from filesystem for test)
    let stored_original_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(stored_original_content, original_content);

    // Simulate external modification
    fs::write(&file_path, external_content).unwrap();

    // Attempt to save edited content with original content should fail
    let save_result =
        save_note_with_content_check(file_name, edited_content, &stored_original_content);
    assert!(
        save_result.is_err(),
        "Save after external change should fail"
    );

    // Verify file contains external content, not edited content
    let final_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(final_content, external_content);
    assert_ne!(final_content, edited_content);

    // Cleanup
    let _ = fs::remove_file(&file_path);
}
