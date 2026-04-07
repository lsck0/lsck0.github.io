#![allow(clippy::needless_return)]

//! Frontmatter parsing for markdown files.
//!
//! Extracts YAML-like key: value pairs between `---` delimiters.
//! Used by both the proc macro (compile-time) and the indexer (post-build).

use std::collections::HashMap;

// ============================================================
// Parsing
// ============================================================

/// Parse frontmatter from a markdown file's content.
/// Returns (metadata map, body text after the closing `---`).
pub fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
    let mut metadata = HashMap::new();

    if !content.starts_with("---") {
        return (metadata, content.to_string());
    }

    let after_first = &content[3..];
    let end = after_first.find("---").unwrap_or(after_first.len());
    let frontmatter = &after_first[..end];
    let body = after_first[end + 3..].trim_start().to_string();

    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();
            metadata.insert(key, value);
        }
    }

    return (metadata, body);
}

// ============================================================
// Metadata helpers
// ============================================================

/// Get a metadata value as a string reference.
pub fn meta_str<'a>(metadata: &'a HashMap<String, String>, key: &str) -> &'a str {
    return metadata.get(key).map(|v| v.as_str()).unwrap_or("");
}

/// Check if a metadata field matches a specific value.
pub fn meta_is(metadata: &HashMap<String, String>, key: &str, expected: &str) -> bool {
    return metadata.get(key).is_some_and(|v| v == expected);
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_frontmatter() {
        let content = "---\ntitle: Hello\ntags: a, b\n---\nBody text here.";
        let (meta, body) = parse_frontmatter(content);
        assert_eq!(meta.get("title").unwrap(), "Hello");
        assert_eq!(meta.get("tags").unwrap(), "a, b");
        assert_eq!(body, "Body text here.");
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "Just some text.";
        let (meta, body) = parse_frontmatter(content);
        assert!(meta.is_empty());
        assert_eq!(body, "Just some text.");
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = "---\n---\nBody.";
        let (meta, body) = parse_frontmatter(content);
        assert!(meta.is_empty());
        assert_eq!(body, "Body.");
    }
}
