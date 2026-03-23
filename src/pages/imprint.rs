use leptos::prelude::*;

#[component]
pub fn ImprintPage() -> impl IntoView {
    return super::prose_page("imprint", include_str!("../../content/imprint.md"));
}
