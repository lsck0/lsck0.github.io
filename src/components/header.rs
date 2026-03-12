#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_router::components::A;

use super::SidebarState;

#[component]
pub fn Header() -> impl IntoView {
    let sidebar_state = use_context::<SidebarState>();

    let toggle_sidebar = move |_| {
        if let Some(state) = sidebar_state {
            state.mobile_open.update(|open| *open = !*open);
        }
    };

    return view! {
        <header class="site-header">
            <button class="sidebar-toggle" on:click=toggle_sidebar title="Menu">
                {"\u{2630}"}
            </button>
            <A href="/" attr:class="site-name">
                {"\u{03bb}"}
            </A>
            <nav class="site-nav">
                <A href="/about">"about"</A>
                <A href="/blog">"blog"</A>
                <A href="/projects">"projects"</A>
                <A href="/publications">"publications"</A>
            </nav>
        </header>
    };
}
