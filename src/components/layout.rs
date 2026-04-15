use leptos::prelude::*;

use super::{
    SidebarState, footer::Footer, header::Header, pinned_panel::PinnedPanel, post_search::GlobalSearch,
    sidebar::Sidebar,
};

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let state = use_context::<SidebarState>().expect("SidebarState context");

    return view! {
        <Header />
        <GlobalSearch />
        <div
            class="sidebar-overlay"
            class:visible=move || state.is_mobile_open.get()
            on:click=move |_| state.is_mobile_open.set(false)
        />
        <div class="page-layout">
            <Sidebar />
            <main>{children()}</main>
            <PinnedPanel />
        </div>
        <Footer />
    };
}
