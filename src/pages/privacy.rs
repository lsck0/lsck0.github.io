use leptos::prelude::*;
use leptos_meta::*;

use crate::{
    components::{layout::Layout, render::markdown_to_html},
    models::meta::META,
};

const PRIVACY_CONTENT: &str = include_str!("../../content/privacy.md");

#[component]
pub fn PrivacyPage() -> impl IntoView {
    let rendered = markdown_to_html(PRIVACY_CONTENT);

    return view! {
        <Title text=format!("\u{03bb} {} \u{2014} privacy", META.title) />
        <Layout>
            <div class="prose-page content" inner_html=rendered.html />
        </Layout>
    };
}
