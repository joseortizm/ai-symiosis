//! Note Rendering Unit Tests
//!
//! Tests for note content rendering functionality.

use crate::*;

#[test]
fn test_render_markdown_note() {
    let markdown_content = "# Hello World\n\nThis is **bold** text.";
    let result = render_note("test.md", markdown_content);

    assert!(result.contains("<h1>"));
    assert!(result.contains("Hello World"));
    assert!(result.contains("<strong>"));
    assert!(result.contains("bold"));
}

#[test]
fn test_render_plain_text_note() {
    let text_content = "This is plain text with <script>alert('xss')</script>";
    let result = render_note("test.txt", text_content);

    assert!(result.starts_with("<pre>"));
    assert!(result.ends_with("</pre>"));
    assert!(result.contains("&lt;script&gt;"));
    assert!(!result.contains("<script>"));
}

#[test]
fn test_render_note_file_extension_detection() {
    let content = "# Test";

    assert!(render_note("test.md", content).contains("<h1>"));
    assert!(render_note("test.markdown", content).contains("<h1>"));

    assert!(render_note("test.txt", content).starts_with("<pre>"));
    assert!(render_note("test.rs", content).starts_with("<pre>"));
    assert!(render_note("no-extension", content).starts_with("<pre>"));
}
