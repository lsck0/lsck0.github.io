use std::collections::BTreeSet;

use leptos::prelude::*;

use crate::{components::listing::ProjectStatus, models::post::POSTS};

pub mod footer;
pub mod header;
pub mod layout;
pub mod listing;
pub mod pinned_panel;
pub mod post_search;
pub mod render;
pub mod scramble;
pub mod sidebar;
pub mod storage;

// ============================================================
// Sidebar State
// ============================================================

#[derive(Clone, Copy)]
pub struct SidebarState {
    pub collapsed_folders: RwSignal<BTreeSet<String>>,
    pub is_mobile_open: RwSignal<bool>,
    pub is_desktop_open: RwSignal<bool>,
    pub is_blog_open: RwSignal<bool>,
    pub is_projects_open: RwSignal<bool>,
    pub collapsed_project_groups: RwSignal<BTreeSet<String>>,
    pub is_publications_open: RwSignal<bool>,
}

impl Default for SidebarState {
    fn default() -> Self {
        return Self::new();
    }
}

impl SidebarState {
    pub fn new() -> Self {
        let mut initially_collapsed_folders = BTreeSet::new();
        for post in POSTS.iter() {
            if !post.folder().is_empty() {
                let mut folder_prefix = String::new();
                for (segment_index, segment) in post.folder().split('/').enumerate() {
                    if segment_index > 0 {
                        folder_prefix.push('/');
                    }
                    folder_prefix.push_str(segment);
                    initially_collapsed_folders.insert(folder_prefix.clone());
                }
            }
        }

        let mut initially_collapsed_project_groups = BTreeSet::new();
        initially_collapsed_project_groups.insert("private".to_string());
        initially_collapsed_project_groups.insert("professional".to_string());
        for status in ProjectStatus::all() {
            initially_collapsed_project_groups.insert(format!("private-{}", status.id()));
            initially_collapsed_project_groups.insert(format!("professional-{}", status.id()));
        }

        return Self {
            collapsed_folders: RwSignal::new(initially_collapsed_folders),
            is_mobile_open: RwSignal::new(false),
            is_desktop_open: RwSignal::new(true),
            is_blog_open: RwSignal::new(true),
            is_projects_open: RwSignal::new(false),
            collapsed_project_groups: RwSignal::new(initially_collapsed_project_groups),
            is_publications_open: RwSignal::new(false),
        };
    }
}

// ============================================================
// Theme Toggle
// ============================================================

const RERENDER_MERMAID_AND_GISCUS_JS: &str = r#"
    if (window.mermaid && window.getMermaidConfig) {
        var config = getMermaidConfig("THEME_PLACEHOLDER");
        mermaid.initialize(config);
        var nodes = document.querySelectorAll(".mermaid[data-source]");
        nodes.forEach(function (node) {
            node.removeAttribute("data-processed");
            node.textContent = node.getAttribute("data-source");
        });
        if (nodes.length > 0) mermaid.run({ nodes: nodes });
    }
    var giscusFrame = document.querySelector("iframe.giscus-frame");
    if (giscusFrame) {
        giscusFrame.contentWindow.postMessage(
            { giscus: { setConfig: { theme: "GISCUS_THEME_PLACEHOLDER" } } },
            "https://giscus.app"
        );
    }
"#;

pub fn toggle_theme() {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let Some(html_element) = document.document_element() else {
        return;
    };

    let current_theme = html_element
        .get_attribute("data-theme")
        .unwrap_or_else(|| "light".to_string());
    let new_theme = if current_theme == "light" { "dark" } else { "light" };

    let _ = html_element.set_attribute("data-theme", new_theme);
    if let Ok(Some(local_storage)) = window.local_storage() {
        let _ = local_storage.set_item("theme", new_theme);
    }

    let giscus_theme = giscus_theme_for(new_theme);
    let rerender_script = RERENDER_MERMAID_AND_GISCUS_JS
        .replace("THEME_PLACEHOLDER", new_theme)
        .replace("GISCUS_THEME_PLACEHOLDER", giscus_theme);
    let _ = js_sys::eval(&rerender_script);
}

// ============================================================
// Shared utilities
// ============================================================

/// Map site theme name to Giscus theme name.
pub fn giscus_theme_for(theme: &str) -> &'static str {
    if theme == "light" { "light" } else { "dark_dimmed" }
}

/// Toggle membership of an item in a BTreeSet (insert if absent, remove if present).
pub fn toggle_in_set(set: &mut BTreeSet<String>, item: &str) {
    if !set.remove(item) {
        set.insert(item.to_string());
    }
}
