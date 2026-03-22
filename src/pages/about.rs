use leptos::prelude::*;
use leptos_meta::*;

use crate::{
    components::{
        layout::Layout,
        render::{call_render_post, markdown_to_html},
    },
    models::meta::META,
};

const ABOUT_CONTENT: &str = include_str!("../../content/about.md");

#[component]
pub fn AboutPage() -> impl IntoView {
    let rendered = markdown_to_html(ABOUT_CONTENT);
    call_render_post();

    return view! {
        <Title text=META.page_title("about") />
        <Meta
            name="description"
            content=META.page("about").map(|page| page.description).unwrap_or("")
        />
        <Layout>
            <div id="post-content" class="prose-page content" inner_html=rendered.html />
        </Layout>
    };
}
