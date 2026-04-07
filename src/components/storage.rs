//! Centralized localStorage helpers for all client-side persistent state.
//! Single source of truth — used by blog listing, post pages, study mode, and pinned blocks.

use std::collections::HashMap;

// ============================================================
// Storage access
// ============================================================

fn get_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

// ============================================================
// Bookmarks
// ============================================================

pub fn is_bookmarked(slug: &str) -> bool {
    get_storage()
        .and_then(|s| s.get_item(&format!("bookmark:{slug}")).ok()?)
        .is_some()
}

pub fn set_bookmarked(slug: &str, bookmarked: bool) {
    if let Some(storage) = get_storage() {
        let key = format!("bookmark:{slug}");
        if bookmarked {
            let _ = storage.set_item(&key, "1");
        } else {
            let _ = storage.remove_item(&key);
        }
    }
}

// ============================================================
// Read status
// ============================================================

pub fn is_read(slug: &str) -> bool {
    get_storage()
        .and_then(|s| s.get_item(&format!("read:{slug}")).ok()?)
        .is_some()
}

pub fn mark_read(slug: &str) {
    if let Some(storage) = get_storage() {
        let _ = storage.set_item(&format!("read:{slug}"), &js_sys::Date::now().to_string());
    }
}

pub fn mark_unread(slug: &str) {
    if let Some(storage) = get_storage() {
        let _ = storage.remove_item(&format!("read:{slug}"));
    }
}

// ============================================================
// Study scores (spaced repetition)
// ============================================================

#[derive(Clone, Default, ::serde::Deserialize, ::serde::Serialize)]
pub struct StudyScore {
    pub right: u32,
    pub unsure: u32,
    pub wrong: u32,
}

pub fn read_study_scores() -> HashMap<String, StudyScore> {
    get_storage()
        .and_then(|s| s.get_item("study-scores").ok()?)
        .and_then(|json| ::serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn save_study_scores(scores: &HashMap<String, StudyScore>) {
    if let Some(storage) = get_storage()
        && let Ok(json) = ::serde_json::to_string(scores)
    {
        let _ = storage.set_item("study-scores", &json);
    }
}

// ============================================================
// Pinned blocks
// ============================================================

#[derive(Clone, ::serde::Deserialize, ::serde::Serialize)]
pub struct PinnedBlock {
    pub label: String,
    pub kind: String,
    pub title: String,
    pub number: String,
    /// Stored as rendered HTML (not markdown).
    pub preview: String,
    pub href: String,
}

pub fn read_pinned_blocks() -> Vec<PinnedBlock> {
    get_storage()
        .and_then(|s| s.get_item("pinned-blocks").ok()?)
        .and_then(|json| ::serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn clear_pinned_blocks() {
    if let Some(storage) = get_storage() {
        let _ = storage.set_item("pinned-blocks", "[]");
    }
    dispatch_pinned_changed();
}

pub fn remove_pinned_block(label: &str) {
    let mut blocks = read_pinned_blocks();
    blocks.retain(|b| b.label != label);
    if let Some(storage) = get_storage()
        && let Ok(json) = ::serde_json::to_string(&blocks)
    {
        let _ = storage.set_item("pinned-blocks", &json);
    }
    dispatch_pinned_changed();
}

fn dispatch_pinned_changed() {
    let _ = js_sys::eval("window.dispatchEvent(new CustomEvent('pinned-blocks-changed'))");
}
