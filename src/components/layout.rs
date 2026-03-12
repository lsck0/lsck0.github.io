#![allow(clippy::needless_return)]

use leptos::prelude::*;

use super::{SidebarState, footer::Footer, header::Header, sidebar::Sidebar};

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let state = use_context::<SidebarState>().expect("SidebarState context");

    return view! {
        <Header />
        <div class="page-layout" class:sidebar-open=move || state.mobile_open.get()>
            <Sidebar />
            <main>{children()}</main>
        </div>
        <Footer />
    };
}
