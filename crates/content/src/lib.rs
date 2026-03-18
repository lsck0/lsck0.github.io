#![allow(clippy::needless_return)]

use std::{collections::HashMap, fs, path::Path};

// ============================================================
// Frontmatter field definitions
// ============================================================

pub const REQUIRED_FIELDS: &[&str] = &["title", "date"];
pub const OPTIONAL_FIELDS: &[&str] = &[
    "tags",
    "description",
    "project",
    "publication",
    "series",
    "series_order",
    "draft",
    "sources",
    "toc",
];

// ============================================================
// Parsed post
// ============================================================

pub struct ParsedPost {
    pub slug: String,
    pub folder: String,
    pub metadata: HashMap<String, String>,
    pub body: String,
    pub internal_links: Vec<String>,
    pub external_links: Vec<String>,
}

impl ParsedPost {
    pub fn title(&self) -> &str {
        return self.metadata.get("title").map(|v| v.as_str()).unwrap_or(&self.slug);
    }

    pub fn date(&self) -> &str {
        return self.metadata.get("date").map(|v| v.as_str()).unwrap_or("");
    }

    pub fn tags(&self) -> Vec<&str> {
        return self
            .metadata
            .get("tags")
            .map(|v| v.split(',').map(|t| t.trim()).collect())
            .unwrap_or_default();
    }

    pub fn description(&self) -> &str {
        return self.metadata.get("description").map(|v| v.as_str()).unwrap_or("");
    }

    pub fn is_draft(&self) -> bool {
        return self.metadata.get("draft").is_some_and(|v| v == "true");
    }

    pub fn series(&self) -> Option<&str> {
        return self.metadata.get("series").map(|v| v.as_str());
    }

    pub fn series_order(&self) -> Option<u32> {
        return self.metadata.get("series_order").and_then(|v| v.parse().ok());
    }

    pub fn sources(&self) -> Vec<&str> {
        return self
            .metadata
            .get("sources")
            .map(|v| v.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();
    }
}

// ============================================================
// Directory parsing
// ============================================================

pub fn parse_posts_directory(directory: &Path) -> Vec<ParsedPost> {
    let mut posts = Vec::new();
    walk_directory(directory, directory, &mut posts);
    posts.sort_by(|a, b| b.date().cmp(a.date()));
    return posts;
}

fn walk_directory(directory: &Path, base_directory: &Path, posts: &mut Vec<ParsedPost>) {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|_| panic!("failed to read directory: {}", directory.display()))
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk_directory(&path, base_directory, posts);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            posts.push(parse_markdown_file(&path, base_directory));
        }
    }
}

fn parse_markdown_file(file_path: &Path, base_directory: &Path) -> ParsedPost {
    let mut content =
        fs::read_to_string(file_path).unwrap_or_else(|_| panic!("failed to read: {}", file_path.display()));
    content = content.replace("\r\n", "\n");

    let (metadata, body) = parse_frontmatter(&content, file_path);

    let file_stem = file_path.file_stem().unwrap().to_string_lossy().to_string();
    let folder = file_path
        .parent()
        .unwrap()
        .strip_prefix(base_directory)
        .unwrap()
        .to_string_lossy()
        .to_string();

    let slug = if folder.is_empty() {
        file_stem
    } else {
        format!("{folder}/{file_stem}")
    };

    let internal_links = extract_internal_links(&body);
    let external_links = extract_external_links(&body);

    return ParsedPost {
        slug,
        folder,
        metadata,
        body,
        internal_links,
        external_links,
    };
}

// ============================================================
// Frontmatter parsing
// ============================================================

pub fn parse_frontmatter(content: &str, source_path: &Path) -> (HashMap<String, String>, String) {
    let trimmed = content.trim_start();
    let display = source_path.display();

    let Some(mut remaining) = trimmed.strip_prefix("---\n") else {
        panic!("{display}: missing frontmatter marker.");
    };

    let mut metadata = HashMap::new();
    loop {
        if remaining.trim().starts_with("---\n") {
            remaining = remaining.trim().strip_prefix("---\n").unwrap();
            break;
        }

        let line_end = remaining.find('\n').unwrap_or_else(|| {
            panic!("{display}: missing closing frontmatter marker.");
        });
        let line = &remaining[..line_end];
        let colon = line.find(':').unwrap_or_else(|| {
            panic!("{display}: invalid frontmatter line: \"{line}\".");
        });
        let (key, value) = line.split_at(colon);
        metadata.insert(key.trim().to_string(), value[1..].trim().to_string());
        remaining = &remaining[line_end + 1..];
    }

    for field in REQUIRED_FIELDS {
        if !metadata.contains_key(*field) {
            panic!("{display}: missing required field \"{field}\".");
        }
    }
    for key in metadata.keys() {
        if !REQUIRED_FIELDS.contains(&key.as_str()) && !OPTIONAL_FIELDS.contains(&key.as_str()) {
            panic!("{display}: unknown field \"{key}\".");
        }
    }

    return (metadata, remaining.to_string());
}

// ============================================================
// Link extraction
// ============================================================

pub fn extract_internal_links(body: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut remaining = body;
    while let Some(position) = remaining.find("/blog/") {
        let after_prefix = &remaining[position + 6..];
        let slug_end = after_prefix
            .find([')', ' ', '\n', '"', '<'])
            .unwrap_or(after_prefix.len());
        let slug = &after_prefix[..slug_end];
        // Strip any #fragment from the slug
        let slug = slug.split('#').next().unwrap_or(slug);
        if !slug.is_empty() && !links.contains(&slug.to_string()) {
            links.push(slug.to_string());
        }
        remaining = &remaining[position + 6..];
    }
    return links;
}

pub fn extract_external_links(body: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut remaining = body;
    while let Some(position) = remaining.find("](http") {
        let url_start = position + 2;
        let after_bracket = &remaining[url_start..];
        if let Some(close_paren) = after_bracket.find(')') {
            let url = &after_bracket[..close_paren];
            if !links.contains(&url.to_string()) {
                links.push(url.to_string());
            }
        }
        remaining = &remaining[url_start..];
    }
    return links;
}

// ============================================================
// Transclusion
// ============================================================

pub fn resolve_transclusions(posts: &mut [ParsedPost]) {
    let bodies: HashMap<String, String> = posts.iter().map(|p| (p.slug.clone(), p.body.clone())).collect();
    for post in posts.iter_mut() {
        post.body = replace_transclusion_markers(&post.body, &bodies);
        post.internal_links = extract_internal_links(&post.body);
        post.external_links = extract_external_links(&post.body);
    }
}

fn replace_transclusion_markers(body: &str, all_bodies: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut remaining = body;

    while let Some(marker_start) = remaining.find("![[") {
        result.push_str(&remaining[..marker_start]);
        let after_marker = &remaining[marker_start + 3..];

        if let Some(marker_end) = after_marker.find("]]") {
            let referenced_slug = &after_marker[..marker_end];
            if let Some(referenced_body) = all_bodies.get(referenced_slug) {
                result.push_str(referenced_body);
            } else {
                result.push_str(&remaining[marker_start..marker_start + 3 + marker_end + 2]);
            }
            remaining = &after_marker[marker_end + 2..];
        } else {
            result.push_str(&remaining[marker_start..]);
            remaining = "";
        }
    }
    result.push_str(remaining);

    return result;
}
