use leptos::prelude::*;

use super::{SidebarState, footer::Footer, header::Header, sidebar::Sidebar};

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let sidebar_state = use_context::<SidebarState>().expect("SidebarState context");

    return view! {
        <div class="marquee-container">
            <marquee>
                "WORK IN PROGRESS; ALL CONTENT IS AI GENERATED SLOPPERY FOR QA TESTING"
            </marquee>
        </div>
        <Header />
        <div class="page-layout" class:sidebar-open=move || sidebar_state.is_mobile_open.get()>
            <Sidebar />
            <main>{children()}</main>
        </div>
        <Footer />
    };
}
