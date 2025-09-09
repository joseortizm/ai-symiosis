use html_escape;
use pulldown_cmark::{html, Options, Parser};

pub fn render_note(filename: &str, content: &str) -> String {
    if filename.ends_with(".md") || filename.ends_with(".markdown") {
        // Configure pulldown-cmark options
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);

        // Parse markdown and convert to HTML
        let parser = Parser::new_ext(content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    } else {
        format!("<pre>{}</pre>", html_escape::encode_text(content))
    }
}
