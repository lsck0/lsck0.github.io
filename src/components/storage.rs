#![allow(clippy::needless_return)]

//! Centralized localStorage helpers for bookmark and read state.
//! Single source of truth — used by both blog listing and post pages.

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
