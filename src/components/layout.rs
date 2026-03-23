use leptos::prelude::*;

use super::{footer::Footer, header::Header, pinned_panel::PinnedPanel, post_search::GlobalSearch, sidebar::Sidebar};

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    return view! {
        <div class="marquee-container">
            <marquee>
                "WORK IN PROGRESS; ALL CONTENT IS AI GENERATED SLOPPERY FOR QA TESTING"
            </marquee>
        </div>
        <Header />
        <GlobalSearch />
        <div class="page-layout">
            <Sidebar />
            <main>{children()}</main>
            <PinnedPanel />
        </div>
        <Footer />
    };
}
