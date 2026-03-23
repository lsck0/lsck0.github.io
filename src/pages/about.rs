use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    return super::prose_page("about", include_str!("../../content/about.md"));
}
