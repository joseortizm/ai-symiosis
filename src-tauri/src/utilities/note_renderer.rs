use crate::core::errors::{AppError, AppResult};
use html_escape;
use once_cell::sync::Lazy;
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;

static URL_REGEX: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r#"(?i)\b(https?://[^\s<>"'`()\[\]{}]+)\b"#));

pub(crate) fn linkify_urls_in_html(html: &str) -> AppResult<String> {
    let url_regex = URL_REGEX
        .as_ref()
        .map_err(|e| AppError::SearchQuery(format!("Failed to compile URL regex: {}", e)))?;

    // More sophisticated check: avoid URLs that are already inside <a> tags
    let result = url_regex
        .replace_all(html, |caps: &regex::Captures| {
            let url = &caps[1];
            let match_start = match caps.get(1) {
                Some(m) => m.start(),
                None => return url.to_string(),
            };

            // Check if this URL is already inside an <a> tag by looking backwards for unclosed <a>
            let before_match = &html[..match_start];
            let last_a_open = before_match.rfind("<a ");
            let last_a_close = before_match.rfind("</a>");

            // If we found an <a> tag that hasn't been closed, don't linkify
            match (last_a_open, last_a_close) {
                (Some(open_pos), Some(close_pos)) if open_pos > close_pos => {
                    // There's an unclosed <a> tag before this URL
                    url.to_string()
                }
                (Some(_), None) => {
                    // There's an <a> tag with no closing tag before this URL
                    url.to_string()
                }
                _ => {
                    // No unclosed <a> tag, safe to linkify
                    format!(
                        r#"<a href="{}" target="_blank" rel="noopener noreferrer">{}</a>"#,
                        url, url
                    )
                }
            }
        })
        .to_string();

    Ok(result)
}

pub fn render_note(filename: &str, content: &str) -> String {
    if filename.ends_with(".md") || filename.ends_with(".markdown") {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        match linkify_urls_in_html(&html_output) {
            Ok(result) => result,
            Err(e) => {
                crate::logging::log(
                    "WARN",
                    &format!("URL linkification failed: {}", e),
                    Some("render_note"),
                );
                html_output // Return original HTML if linkification fails
            }
        }
    } else {
        let escaped = html_escape::encode_text(content);
        match linkify_urls_in_html(&escaped) {
            Ok(linkified) => format!("<pre>{}</pre>", linkified),
            Err(e) => {
                crate::logging::log(
                    "WARN",
                    &format!("URL linkification failed: {}", e),
                    Some("render_note"),
                );
                format!("<pre>{}</pre>", escaped) // Return original escaped content if linkification fails
            }
        }
    }
}
