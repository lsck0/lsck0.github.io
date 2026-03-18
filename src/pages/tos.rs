use leptos::prelude::*;
use leptos_meta::*;

use crate::{
    components::{layout::Layout, render::markdown_to_html},
    models::meta::META,
};

const TOS_CONTENT: &str = include_str!("../../content/tos.md");

#[component]
pub fn TosPage() -> impl IntoView {
    let rendered = markdown_to_html(TOS_CONTENT);

    return view! {
        <Title text=format!("\u{03bb} {} \u{2014} terms of service", META.title) />
        <Layout>
            <div class="prose-page content" inner_html=rendered.html />
        </Layout>
    };
}
