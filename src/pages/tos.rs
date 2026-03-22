use leptos::prelude::*;
use leptos_meta::*;

use crate::{
    components::{
        layout::Layout,
        render::{call_render_post, markdown_to_html},
    },
    models::meta::META,
};

const TOS_CONTENT: &str = include_str!("../../content/tos.md");

#[component]
pub fn TosPage() -> impl IntoView {
    let rendered = markdown_to_html(TOS_CONTENT);
    call_render_post();

    return view! {
        <Title text=META.page_title("tos") />
        <Meta
            name="description"
            content=META.page("tos").map(|page| page.description).unwrap_or("")
        />
        <Layout>
            <div id="post-content" class="prose-page content" inner_html=rendered.html />
        </Layout>
    };
}
