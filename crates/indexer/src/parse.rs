#![allow(clippy::needless_return, dead_code)]

//! Post reading and parsing for the indexer.
//!
//! Reads markdown files from the content directory, parses frontmatter,
//! extracts links, and resolves transclusions.
//! Uses `ir::frontmatter` for the shared frontmatter parser.

use std::{collections::HashMap, fs, path::Path, process::Command};

use anyhow::{Context, Result};

// ============================================================
// Parsed post
// ============================================================

pub struct ContentPost {
    pub slug: String,
    pub folder: String,
    pub metadata: HashMap<String, String>,
    pub body: String,
    pub internal_links: Vec<String>,
    pub external_links: Vec<String>,
}

impl ContentPost {
    pub fn title(&self) -> &str {
        return self.metadata.get("title").map(|v| v.as_str()).unwrap_or(&self.slug);
    }

    pub fn date(&self) -> &str {
        return self.metadata.get("created").map(|v| v.as_str()).unwrap_or("");
    }

    pub fn last_edited(&self) -> &str {
        return self.metadata.get("last_edited").map(|v| v.as_str()).unwrap_or("");
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

pub fn parse_posts_directory(directory: &Path) -> Vec<ContentPost> {
    let mut posts = Vec::new();
    walk_directory(directory, directory, &mut posts).unwrap_or_else(|e| panic!("failed to parse posts: {e:#}"));
    posts.sort_by(|a, b| b.date().cmp(a.date()));
    return posts;
}

fn walk_directory(directory: &Path, base_directory: &Path, posts: &mut Vec<ContentPost>) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("failed to read directory: {}", directory.display()))?
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk_directory(&path, base_directory, posts)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let post = parse_markdown_file(&path, base_directory)
                .with_context(|| format!("failed to parse: {}", path.display()))?;
            posts.push(post);
        }
    }
    return Ok(());
}

fn parse_markdown_file(file_path: &Path, base_directory: &Path) -> Result<ContentPost> {
    let mut content =
        fs::read_to_string(file_path).with_context(|| format!("failed to read: {}", file_path.display()))?;
    content = content.replace("\r\n", "\n");

    let (mut metadata, body) = ir::frontmatter::parse_frontmatter(&content);

    // Inject git-derived dates
    let (created, last_edited) = git_file_dates(file_path);
    metadata.insert("created".to_string(), created);
    metadata.insert("last_edited".to_string(), last_edited);

    let file_stem = file_path
        .file_stem()
        .context("file has no stem")?
        .to_string_lossy()
        .to_string();
    let folder = file_path
        .parent()
        .context("file has no parent")?
        .strip_prefix(base_directory)
        .context("file not under base directory")?
        .to_string_lossy()
        .to_string();

    let slug = if folder.is_empty() {
        file_stem
    } else {
        format!("{folder}/{file_stem}")
    };

    let internal_links = extract_internal_links(&body);
    let external_links = extract_external_links(&body);

    return Ok(ContentPost {
        slug,
        folder,
        metadata,
        body,
        internal_links,
        external_links,
    });
}

// ============================================================
// Git date extraction
// ============================================================

fn git_file_dates(file_path: &Path) -> (String, String) {
    let path_str = file_path.to_string_lossy().to_string();

    let last_edited = Command::new("git")
        .args(["log", "-1", "--format=%cd", "--date=short", "--", &path_str])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|date| date.trim().to_string())
        .filter(|date| !date.is_empty());

    let created = Command::new("git")
        .args([
            "log",
            "--diff-filter=A",
            "--follow",
            "--format=%cd",
            "--date=short",
            "--",
            &path_str,
        ])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|date| date.trim().lines().last().unwrap_or("").trim().to_string())
        .filter(|date| !date.is_empty());

    let fallback = || {
        fs::metadata(file_path)
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|time| {
                let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
                let secs = duration.as_secs();
                let days = secs / 86400;
                let years = 1970 + days / 365;
                Some(format!("{years}-01-01"))
            })
            .unwrap_or_else(|| "2026-01-01".to_string())
    };

    let created_date = created.unwrap_or_else(fallback);
    let last_edited_date = last_edited.unwrap_or_else(|| created_date.clone());

    return (created_date, last_edited_date);
}

// ============================================================
// Link extraction
// ============================================================

fn extract_internal_links(body: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut remaining = body;
    while let Some(position) = remaining.find("/blog/") {
        let after_prefix = &remaining[position + 6..];
        let slug_end = after_prefix
            .find([')', ' ', '\n', '"', '<'])
            .unwrap_or(after_prefix.len());
        let slug = &after_prefix[..slug_end];
        let slug = slug.split('#').next().unwrap_or(slug);
        if !slug.is_empty() && !links.contains(&slug.to_string()) {
            links.push(slug.to_string());
        }
        remaining = &remaining[position + 6..];
    }
    return links;
}

fn extract_external_links(body: &str) -> Vec<String> {
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

pub fn resolve_transclusions(posts: &mut [ContentPost]) {
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
