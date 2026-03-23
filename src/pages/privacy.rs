use leptos::prelude::*;

#[component]
pub fn PrivacyPage() -> impl IntoView {
    return super::prose_page("privacy", include_str!("../../content/privacy.md"));
}
