use nucleo_matcher::{Config, Matcher, Utf32Str};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub filename: String,
    pub title: String,
    pub score: u32,
    pub match_type: MatchType,
    pub modified: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchType {
    ExactTitle = 4,
    PrefixTitle = 3,
    FuzzyTitle = 2,
    Content = 1,
}

#[derive(Debug, Clone)]
struct SearchCandidate {
    filename: String,
    title: String,
    content: String,
    modified: i64,
}

pub struct HybridSearcher {
    conn: Connection,
}

impl HybridSearcher {
    pub fn new(db_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    fn extract_title_from_filename(filename: &str) -> String {
        // Remove extension and convert underscores/hyphens to spaces
        let stem = filename
            .trim_end_matches(".md")
            .trim_end_matches(".txt")
            .trim_end_matches(".markdown")
            .replace('_', " ")
            .replace('-', " ");
        stem
    }

    fn extract_title_from_content(content: &str) -> Option<String> {
        // Try to get first non-empty line as title
        content
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| {
                // Remove markdown header markers
                line.trim_start_matches('#').trim().to_string()
            })
            .filter(|title| !title.is_empty())
    }

    pub fn search(&self, query: &str, max_results: usize) -> Result<Vec<String>, String> {
        if query.trim().is_empty() {
            return self.get_recent_notes(max_results);
        }

        // Step 1: Get candidates from SQLite for fast filtering
        let candidates = self.get_candidates_from_sqlite(query)?;

        // Step 2: Score and rank candidates
        let mut results = Vec::new();

        for candidate in candidates {
            if let Some(result) = self.score_candidate(&candidate, query) {
                results.push(result);
            }
        }

        // Step 3: Sort by custom ranking rules
        results.sort_by(|a, b| self.compare_results(a, b));
        results.truncate(max_results);

        // Return filenames to match existing API
        Ok(results.into_iter().map(|r| r.filename).collect())
    }

    pub fn search_with_metadata(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, String> {
        if query.trim().is_empty() {
            return self.get_recent_notes_with_metadata(max_results);
        }

        let candidates = self.get_candidates_from_sqlite(query)?;
        let mut results = Vec::new();

        for candidate in candidates {
            if let Some(result) = self.score_candidate(&candidate, query) {
                results.push(result);
            }
        }

        results.sort_by(|a, b| self.compare_results(a, b));
        results.truncate(max_results);

        Ok(results)
    }

    fn get_candidates_from_sqlite(&self, query: &str) -> Result<Vec<SearchCandidate>, String> {
        // Use SQLite FTS for initial fast filtering
        let fts_pattern = if query.contains(' ') {
            query
                .split_whitespace()
                .map(|word| format!("{}*", word.replace('"', "")))
                .collect::<Vec<_>>()
                .join(" OR ")
        } else {
            format!("{}*", query.replace('"', ""))
        };

        let mut stmt = self
            .conn
            .prepare(
                "SELECT filename, content, modified FROM notes
                 WHERE notes MATCH ?
                 ORDER BY rank
                 LIMIT 500",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![fts_pattern], |row| {
                let filename: String = row.get(0)?;
                let content: String = row.get(1)?;
                let modified: i64 = row.get(2)?;

                let title = Self::extract_title_from_content(&content)
                    .unwrap_or_else(|| Self::extract_title_from_filename(&filename));

                Ok(SearchCandidate {
                    filename,
                    title,
                    content,
                    modified,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    fn score_candidate(&self, candidate: &SearchCandidate, query: &str) -> Option<SearchResult> {
        let query_lower = query.to_lowercase();
        let title_lower = candidate.title.to_lowercase();
        let filename_lower = candidate.filename.to_lowercase();

        // Check for title matches first (highest priority)
        if let Some((score, match_type)) =
            self.score_title_match(&title_lower, &filename_lower, &query_lower)
        {
            return Some(SearchResult {
                filename: candidate.filename.clone(),
                title: candidate.title.clone(),
                score,
                match_type,
                modified: candidate.modified,
            });
        }

        // Fallback to content matching
        if let Some(score) = self.score_content_match(&candidate.content, query) {
            return Some(SearchResult {
                filename: candidate.filename.clone(),
                title: candidate.title.clone(),
                score,
                match_type: MatchType::Content,
                modified: candidate.modified,
            });
        }

        None
    }

    fn score_title_match(
        &self,
        title_lower: &str,
        filename_lower: &str,
        query_lower: &str,
    ) -> Option<(u32, MatchType)> {
        // Check both title and filename for matches
        for (text, boost) in [(title_lower, 100), (filename_lower, 50)] {
            // Exact match
            if text == query_lower {
                return Some((1000 + boost, MatchType::ExactTitle));
            }

            // Prefix match
            if text.starts_with(query_lower) {
                return Some((800 + boost, MatchType::PrefixTitle));
            }

            // Word boundary prefix match
            if text
                .split_whitespace()
                .any(|word| word.starts_with(query_lower))
            {
                return Some((700 + boost, MatchType::PrefixTitle));
            }

            // Contains match
            if text.contains(query_lower) {
                return Some((600 + boost, MatchType::FuzzyTitle));
            }
        }

        // Use fuzzy matching for title
        if let Some(score) = self.fuzzy_match(title_lower, query_lower) {
            if score > 50 {
                // Threshold for fuzzy title matches
                return Some((score + 200, MatchType::FuzzyTitle));
            }
        }

        None
    }

    fn score_content_match(&self, content: &str, query: &str) -> Option<u32> {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();

        // Simple scoring for content matches
        if content_lower.contains(&query_lower) {
            // Count occurrences for better scoring
            let count = content_lower.matches(&query_lower).count() as u32;
            Some(50 + count * 10)
        } else {
            self.fuzzy_match(&content_lower, &query_lower)
        }
    }

    fn fuzzy_match(&self, text: &str, query: &str) -> Option<u32> {
        let text_chars: Vec<char> = text.chars().collect();
        let query_chars: Vec<char> = query.chars().collect();

        if query_chars.is_empty() {
            return Some(0);
        }

        let mut text_idx = 0;
        let mut query_idx = 0;
        let mut score = 0u32;
        let mut consecutive = 0u32;

        while text_idx < text_chars.len() && query_idx < query_chars.len() {
            if text_chars[text_idx]
                .to_lowercase()
                .eq(query_chars[query_idx].to_lowercase())
            {
                score += 1 + consecutive;
                consecutive += 1;
                query_idx += 1;
            } else {
                consecutive = 0;
            }
            text_idx += 1;
        }

        if query_idx == query_chars.len() {
            // All query characters matched
            let match_ratio = score as f32 / query_chars.len() as f32;
            Some((match_ratio * 100.0) as u32)
        } else {
            None
        }
    }

    fn compare_results(&self, a: &SearchResult, b: &SearchResult) -> Ordering {
        // 1. Match type priority (title matches beat content matches)
        match b.match_type.cmp(&a.match_type) {
            Ordering::Equal => {}
            other => return other,
        }

        // 2. Score within same match type
        match b.score.cmp(&a.score) {
            Ordering::Equal => {}
            other => return other,
        }

        // 3. Newer notes first
        match b.modified.cmp(&a.modified) {
            Ordering::Equal => {}
            other => return other,
        }

        // 4. Alphabetical by title
        a.title.cmp(&b.title)
    }

    fn get_recent_notes(&self, max_results: usize) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT filename FROM notes ORDER BY modified DESC LIMIT ?")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([max_results], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    fn get_recent_notes_with_metadata(
        &self,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT filename, content, modified FROM notes ORDER BY modified DESC LIMIT ?")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([max_results], |row| {
                let filename: String = row.get(0)?;
                let content: String = row.get(1)?;
                let modified: i64 = row.get(2)?;

                let title = Self::extract_title_from_content(&content)
                    .unwrap_or_else(|| Self::extract_title_from_filename(&filename));

                Ok(SearchResult {
                    filename,
                    title,
                    score: 0,
                    match_type: MatchType::Content,
                    modified,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
}

// Convenience functions for use in main lib
pub fn search_notes_hybrid(
    query: &str,
    db_path: &Path,
    max_results: usize,
) -> Result<Vec<String>, String> {
    let searcher = HybridSearcher::new(db_path).map_err(|e| e.to_string())?;
    searcher.search(query, max_results)
}

// pub fn search_notes_with_metadata(
//     query: &str,
//     db_path: &Path,
//     max_results: usize,
// ) -> Result<Vec<SearchResult>, String> {
//     let searcher = HybridSearcher::new(db_path).map_err(|e| e.to_string())?;
//     searcher.search_with_metadata(query, max_results)
// }
