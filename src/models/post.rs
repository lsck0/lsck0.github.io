use macros::include_posts;

pub static POSTS: &[Post] = include_posts!();

// ============================================================
// Date formatting
// ============================================================

fn format_date(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }
    let year = parts[0];
    let month = parts[1];
    let day = parts[2];
    return format!("{day}.{month}.{year}");
}

// ============================================================
// Labeled blocks
// ============================================================

pub struct LabeledBlock {
    pub label: &'static str,
    pub kind: &'static str,
    pub title: &'static str,
    pub number: &'static str,
    pub content: &'static str,
}

// ============================================================
// Post
// ============================================================

pub struct Post {
    pub slug: &'static str,
    pub folder: &'static str,
    pub metadata: &'static [(&'static str, &'static str)],
    pub body: &'static str,
    pub internal_links: &'static [&'static str],
    pub external_links: &'static [&'static str],
    pub sources: &'static [&'static str],
    pub labeled_blocks: &'static [LabeledBlock],
}

impl Post {
    fn metadata_field(&self, key: &str) -> Option<&'static str> {
        return self
            .metadata
            .iter()
            .find(|(field_key, _)| *field_key == key)
            .map(|(_, value)| *value);
    }

    pub fn title(&self) -> &'static str {
        return self.metadata_field("title").unwrap_or(self.slug);
    }

    pub fn date(&self) -> &'static str {
        return self.metadata_field("created").unwrap_or("");
    }

    pub fn last_edited(&self) -> &'static str {
        return self.metadata_field("last_edited").unwrap_or("");
    }

    pub fn tags(&self) -> Vec<&'static str> {
        return self
            .metadata_field("tags")
            .map(|value| value.split(',').map(|tag_name| tag_name.trim()).collect())
            .unwrap_or_default();
    }

    pub fn description(&self) -> &'static str {
        return self.metadata_field("description").unwrap_or("");
    }

    pub fn project(&self) -> Option<&'static str> {
        return self.metadata_field("project");
    }

    pub fn publication(&self) -> Option<&'static str> {
        return self.metadata_field("publication");
    }

    pub fn series(&self) -> Option<&'static str> {
        return self.metadata_field("series");
    }

    pub fn series_order(&self) -> Option<u32> {
        return self.metadata_field("series_order").and_then(|value| value.parse().ok());
    }

    pub fn is_draft(&self) -> bool {
        return self.metadata_field("draft").is_some_and(|value| value == "true");
    }

    pub fn toc(&self) -> bool {
        return self.metadata_field("toc").is_some_and(|value| value == "true");
    }

    pub fn href(&self) -> String {
        return format!("/blog/{}", self.slug);
    }

    pub fn date_formatted(&self) -> String {
        return format_date(self.date());
    }

    pub fn last_edited_formatted(&self) -> String {
        return format_date(self.last_edited());
    }

    /// Concatenated text from all labeled block titles and labels, for search indexing.
    pub fn labeled_block_text(&self) -> String {
        return self
            .labeled_blocks
            .iter()
            .flat_map(|block| [block.title, block.label, block.kind])
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
        let mut posts: Vec<&Post> = POSTS.iter().filter(|post| post.series() == Some(series_name)).collect();
        posts.sort_by_key(|post| post.series_order().unwrap_or(0));
        return posts;
    }

    /// Posts that this post links to (pre-computed at build time).
    pub fn outgoing_links(&self) -> Vec<&'static Post> {
        return self
            .internal_links
            .iter()
            .filter_map(|slug| POSTS.iter().find(|post| post.slug == *slug))
            .collect();
    }

    /// Posts that link to this post (scanned at runtime from all posts' internal_links).
    pub fn incoming_links(&self) -> Vec<&'static Post> {
        return POSTS
            .iter()
            .filter(|post| post.slug != self.slug && post.internal_links.contains(&self.slug))
            .collect();
    }
}
