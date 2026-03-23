use leptos::prelude::*;

#[component]
pub fn TosPage() -> impl IntoView {
    return super::prose_page("tos", include_str!("../../content/tos.md"));
}
