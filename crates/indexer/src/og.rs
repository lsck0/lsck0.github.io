#![allow(clippy::needless_return)]

use std::{collections::HashMap, fs, path::Path, time::Duration};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::parse::ContentPost;

// ============================================================
// OG metadata types
// ============================================================

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct OgMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

// ============================================================
// Collect external URLs from posts
// ============================================================

pub fn collect_external_urls(posts: &[&ContentPost]) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    for post in posts {
        for url in &post.external_links {
            if !urls.contains(url) {
                urls.push(url.clone());
            }
        }
    }
    return urls;
}

// ============================================================
// Fetch and cache OG metadata
// ============================================================

pub fn build_og_metadata(posts: &[&ContentPost], cache_path: &Path) -> Result<HashMap<String, OgMetadata>> {
    let urls = collect_external_urls(posts);
    if urls.is_empty() {
        return Ok(HashMap::new());
    }

    // Load existing cache
    let mut cache: HashMap<String, OgMetadata> = if cache_path.exists() {
        let content = fs::read_to_string(cache_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Fetch missing entries
    let mut fetched = 0;
    for url in &urls {
        if cache.contains_key(url) {
            continue;
        }
        match fetch_og_metadata(url) {
            Ok(meta) => {
                println!("  OG fetched: {url}");
                cache.insert(url.clone(), meta);
                fetched += 1;
            }
            Err(err) => {
                println!("  OG failed:  {url} ({err})");
                cache.insert(url.clone(), OgMetadata::default());
            }
        }
    }

    if fetched > 0 {
        // Write updated cache
        let json = serde_json::to_string_pretty(&cache).context("failed to serialize OG cache")?;
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(cache_path, &json).context("failed to write OG cache")?;
    }

    // Return only entries for URLs that appear in current posts
    let result: HashMap<String, OgMetadata> = cache
        .into_iter()
        .filter(|(url, _)| urls.contains(url))
        .filter(|(_, meta)| meta.title.is_some() || meta.description.is_some())
        .collect();

    return Ok(result);
}

// ============================================================
// Fetch OG metadata from a single URL
// ============================================================

fn fetch_og_metadata(url: &str) -> Result<OgMetadata> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .into();

    let body: String = agent
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; BlogIndexer/1.0)")
        .call()
        .context("HTTP request failed")?
        .body_mut()
        .read_to_string()
        .context("failed to read response body")?;

    // Only parse the <head> section for efficiency
    let head_end = body.find("</head>").unwrap_or(body.len().min(16_000));
    let head = &body[..head_end];

    let title = extract_meta_content(head, "og:title")
        .or_else(|| extract_meta_content(head, "twitter:title"))
        .or_else(|| extract_title_tag(head));
    let description = extract_meta_content(head, "og:description")
        .or_else(|| extract_meta_content(head, "twitter:description"))
        .or_else(|| extract_meta_name_content(head, "description"));
    let image = extract_meta_content(head, "og:image").or_else(|| extract_meta_content(head, "twitter:image"));

    return Ok(OgMetadata {
        title,
        description,
        image,
    });
}

// ============================================================
// HTML meta tag extraction
// ============================================================

fn extract_meta_content(html: &str, property: &str) -> Option<String> {
    // Match: <meta property="og:title" content="..."/>
    let pattern = format!("property=\"{property}\"");
    extract_content_near_pattern(html, &pattern)
}

fn extract_meta_name_content(html: &str, name: &str) -> Option<String> {
    // Match: <meta name="description" content="..."/>
    let pattern = format!("name=\"{name}\"");
    extract_content_near_pattern(html, &pattern)
}

fn extract_content_near_pattern(html: &str, pattern: &str) -> Option<String> {
    let pos = html.find(pattern)?;

    // Find the enclosing <meta ... > tag
    let tag_start = html[..pos].rfind('<')?;
    let tag_end = html[pos..].find('>')? + pos;
    let tag = &html[tag_start..=tag_end];

    // Extract content="..." from the tag
    let content_start = tag.find("content=\"")? + 9;
    let content_end = tag[content_start..].find('"')? + content_start;
    let value = &tag[content_start..content_end];

    if value.is_empty() {
        return None;
    }

    return Some(html_unescape(value));
}

fn extract_title_tag(html: &str) -> Option<String> {
    let start = html.find("<title")? + 6;
    let after_open = html[start..].find('>')? + start + 1;
    let end = html[after_open..].find("</title>")? + after_open;
    let title = html[after_open..end].trim();
    if title.is_empty() {
        return None;
    }
    return Some(html_unescape(title));
}

fn html_unescape(input: &str) -> String {
    return input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");
}
