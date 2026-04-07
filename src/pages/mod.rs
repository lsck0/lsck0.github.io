use leptos::prelude::*;
use leptos_meta::*;

use crate::{
    components::{layout::Layout, render::call_render_post},
    models::meta::META,
};

pub mod about;
pub mod blog;
pub mod graph;
pub mod home;
pub mod imprint;
pub mod not_found;
pub mod post;
pub mod privacy;
pub mod projects;
pub mod publications;
pub mod tos;

// ============================================================
// Shared prose page template
// ============================================================

/// Renders a static markdown content page with Layout, Title, and Meta.
/// Uses the IR pipeline so prose pages get the same features as blog posts.
pub fn prose_page(page_key: &'static str, content: &str) -> impl IntoView {
    let (blocks, _) = ir::parse::parse_markdown(content, false);
    let (html, _) = crate::components::render::render_content(&blocks, |_| None);
    call_render_post();

    return view! {
        <Title text=META.page_title(page_key) />
        <Meta
            name="description"
            content=META.page(page_key).map(|page| page.description).unwrap_or("")
        />
        <Layout>
            <div id="post-content" class="prose-page content" inner_html=html />
        </Layout>
    };
}
