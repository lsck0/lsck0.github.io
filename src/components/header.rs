use leptos::prelude::*;
use leptos_router::components::A;

use super::SidebarState;

#[component]
pub fn Header() -> impl IntoView {
    let sidebar_state = use_context::<SidebarState>();

    let toggle_sidebar = move |_| {
        if let Some(state) = sidebar_state {
            state.is_mobile_open.update(|is_open| *is_open = !*is_open);
        }
    };

    return view! {
        <header class="site-header">
            <button class="sidebar-toggle" on:click=toggle_sidebar title="Menu">
                {"\u{2630}"}
            </button>
            <A href="/" attr:class="nav-lambda">
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
