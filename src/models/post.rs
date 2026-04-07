#![allow(clippy::needless_return)]

use std::sync::LazyLock;

use ir::types::{BlockMeta, CitationMeta, Page, SiteData};
use macros::{include_katex_macros, include_posts};

// ============================================================
// Static site data
// ============================================================

static SITE_BYTES: &[u8] = include_posts!();

/// KaTeX macros parsed from `content/macros.tex` at compile time.
pub static KATEX_MACROS_JSON: &str = include_katex_macros!();

static SITE_DATA: LazyLock<SiteData> =
    LazyLock::new(|| postcard::from_bytes(SITE_BYTES).expect("failed to deserialize SiteData"));

pub static POSTS: LazyLock<Vec<Post>> = LazyLock::new(|| SITE_DATA.pages.iter().map(|page| Post { page }).collect());

// ============================================================
// Post wrapper
// ============================================================

pub struct Post {
    pub page: &'static Page,
}

impl Post {
    fn meta(&self, key: &str) -> Option<&str> {
        return self.page.metadata.get(key).map(|v| v.as_str());
    }

    pub fn slug(&self) -> &str {
        return &self.page.slug;
    }

    pub fn folder(&self) -> &str {
        return &self.page.folder;
    }

    pub fn title(&self) -> &str {
        return self.meta("title").unwrap_or(self.slug());
    }

    pub fn date(&self) -> &str {
        return self.meta("created").unwrap_or("");
    }

    pub fn last_edited(&self) -> &str {
        return self.meta("last_edited").unwrap_or("");
    }

    pub fn tags(&self) -> Vec<&str> {
        return self
            .meta("tags")
            .map(|v| v.split(',').map(|t| t.trim()).collect())
            .unwrap_or_default();
    }

    pub fn description(&self) -> &str {
        return self.meta("description").unwrap_or("");
    }

    pub fn project(&self) -> Option<&str> {
        return self.meta("project");
    }

    pub fn publication(&self) -> Option<&str> {
        return self.meta("publication");
    }

    pub fn series(&self) -> Option<&str> {
        return self.meta("series");
    }

    pub fn series_order(&self) -> Option<u32> {
        return self.meta("series_order").and_then(|v| v.parse().ok());
    }

    pub fn is_draft(&self) -> bool {
        return self.meta("draft").is_some_and(|v| v == "true");
    }

    pub fn toc(&self) -> bool {
        return self.meta("toc").is_some_and(|v| v == "true");
    }

    pub fn href(&self) -> String {
        return format!("/blog/{}", self.slug());
    }

    pub fn date_formatted(&self) -> String {
        return format_date(self.date());
    }

    pub fn last_edited_formatted(&self) -> String {
        return format_date(self.last_edited());
    }

    pub fn content(&self) -> &[ir::types::Block] {
        return &self.page.content;
    }

    pub fn blocks(&self) -> &[BlockMeta] {
        return &self.page.blocks;
    }

    pub fn citations(&self) -> &[CitationMeta] {
        return &self.page.citations;
    }

    pub fn internal_links(&self) -> &[String] {
        return &self.page.internal_links;
    }

    pub fn external_links(&self) -> &[String] {
        return &self.page.external_links;
    }

    pub fn sources(&self) -> &[String] {
        return &self.page.sources;
    }

    pub fn labeled_block_text(&self) -> String {
        return self
            .blocks()
            .iter()
            .flat_map(|b| [b.title.as_str(), b.label.as_str(), b.kind.as_str()])
            .collect::<Vec<_>>()
            .join(" ");
    }

    // ============================================================
    // Relations
    // ============================================================

    pub fn series_posts(&self) -> Vec<&'static Post> {
        let Some(series_name) = self.series() else {
            return vec![];
        };
        let mut posts: Vec<&Post> = POSTS.iter().filter(|p| p.series() == Some(series_name)).collect();
        posts.sort_by_key(|p| p.series_order().unwrap_or(0));
        return posts;
    }

    pub fn outgoing_links(&self) -> Vec<&'static Post> {
        return self
            .internal_links()
            .iter()
            .filter_map(|slug| POSTS.iter().find(|p| p.slug() == slug))
            .collect();
    }

    pub fn incoming_links(&self) -> Vec<&'static Post> {
        return POSTS
            .iter()
            .filter(|p| p.slug() != self.slug() && p.internal_links().iter().any(|s| s == self.slug()))
            .collect();
    }
}

// ============================================================
// Date formatting
// ============================================================

fn format_date(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }
    return format!("{}.{}.{}", parts[2], parts[1], parts[0]);
}
