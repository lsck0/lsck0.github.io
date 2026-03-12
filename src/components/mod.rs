#![allow(clippy::needless_return)]

use std::collections::BTreeSet;

use leptos::prelude::*;

use crate::models::post::POSTS;

pub mod footer;
pub mod header;
pub mod layout;
pub mod listing;
pub mod render;
pub mod sidebar;

#[derive(Clone, Copy)]
pub struct SidebarState {
    pub collapsed: RwSignal<BTreeSet<String>>,
    pub mobile_open: RwSignal<bool>,
    pub blog_open: RwSignal<bool>,
}

impl Default for SidebarState {
    fn default() -> Self {
        return Self::new();
    }
}

impl SidebarState {
    pub fn new() -> Self {
        let mut initially_collapsed = BTreeSet::new();
        for post in POSTS {
            if !post.folder.is_empty() {
                let mut prefix = String::new();
                for (i, part) in post.folder.split('/').enumerate() {
                    if i > 0 {
                        prefix.push('/');
                    }
                    prefix.push_str(part);
                    initially_collapsed.insert(prefix.clone());
                }
            }
        }
        return Self {
            collapsed: RwSignal::new(initially_collapsed),
            mobile_open: RwSignal::new(false),
            blog_open: RwSignal::new(true),
        };
    }
}

pub fn toggle_theme() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let html = document.document_element().unwrap();

    let current = html.get_attribute("data-theme").unwrap_or_else(|| "light".to_string());
    let next = if current == "light" { "dark" } else { "light" };

    html.set_attribute("data-theme", next).unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    storage.set_item("theme", next).unwrap();

    // Re-render mermaid diagrams with the updated theme.
    // Mermaid is a JS library, so we call it via eval.
    let _ = js_sys::eval(&format!(
        r#"
        if (window.mermaid && window.getMermaidConfig) {{
            var config = getMermaidConfig("{theme}");
            mermaid.initialize(config);
            var nodes = document.querySelectorAll(".mermaid[data-source]");
            nodes.forEach(function (node) {{
                node.removeAttribute("data-processed");
                node.textContent = node.getAttribute("data-source");
            }});
            if (nodes.length > 0) mermaid.run({{ nodes: nodes }});
        }}
        "#,
        theme = next
    ));
}
