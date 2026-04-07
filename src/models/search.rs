use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};

use super::post::Post;

// ============================================================
// Fuzzy matching
// ============================================================

pub fn fuzzy_score(query: &str, text: &str) -> Option<u32> {
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        nucleo_matcher::pattern::AtomKind::Fuzzy,
    );
    let mut buf = Vec::new();
    let haystack = Utf32Str::new(text, &mut buf);
    return pattern.score(haystack, &mut matcher);
}

// ============================================================
// Post scoring
// ============================================================

/// Score a post against a search query.
/// Weights: title ×3, description ×1, labeled blocks ×2.
/// Returns the best score across all fields, or None if no match.
pub fn score_post(query: &str, post: &Post) -> Option<u32> {
    let title_score = fuzzy_score(query, post.title()).map(|s| s.saturating_mul(3));
    let desc_score = fuzzy_score(query, post.description());
    let block_text = post.labeled_block_text();
    let block_score = fuzzy_score(query, &block_text).map(|s| s.saturating_mul(2));
    return [title_score, desc_score, block_score].into_iter().flatten().max();
}
