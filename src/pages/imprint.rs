use leptos::prelude::*;
use leptos_meta::*;

use crate::{
    components::{
        layout::Layout,
        render::{call_render_post, markdown_to_html},
    },
    models::meta::META,
};

const IMPRINT_CONTENT: &str = include_str!("../../content/imprint.md");

#[component]
pub fn ImprintPage() -> impl IntoView {
    let rendered = markdown_to_html(IMPRINT_CONTENT);
    call_render_post();

    return view! {
        <Title text=META.page_title("imprint") />
        <Meta
            name="description"
            content=META.page("imprint").map(|page| page.description).unwrap_or("")
        />
        <Layout>
            <div id="post-content" class="prose-page content" inner_html=rendered.html />
        </Layout>
    };
}
