use leptos::prelude::*;
use leptos_meta::*;

use crate::{
    components::{
        layout::Layout,
        render::{call_render_post, markdown_to_html},
    },
    models::meta::META,
};

const PRIVACY_CONTENT: &str = include_str!("../../content/privacy.md");

#[component]
pub fn PrivacyPage() -> impl IntoView {
    let rendered = markdown_to_html(PRIVACY_CONTENT);
    call_render_post();

    return view! {
        <Title text=META.page_title("privacy") />
        <Meta
            name="description"
            content=META.page("privacy").map(|page| page.description).unwrap_or("")
        />
        <Layout>
            <div id="post-content" class="prose-page content" inner_html=rendered.html />
        </Layout>
    };
}
