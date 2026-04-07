use leptos::prelude::*;
use leptos_router::components::A;

use super::SidebarState;

const MOBILE_BREAKPOINT: f64 = 768.0;

fn is_mobile() -> bool {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|w| w.as_f64())
        .is_some_and(|w| w <= MOBILE_BREAKPOINT)
}

#[component]
pub fn Header() -> impl IntoView {
    let state = use_context::<SidebarState>().expect("SidebarState context");

    let toggle_sidebar = move |_| {
        if is_mobile() {
            state.is_mobile_open.update(|open| *open = !*open);
        } else {
            state.is_desktop_open.update(|open| *open = !*open);
        }
    };

    return view! {
        <header class="site-header">
            <button class="sidebar-toggle" on:click=toggle_sidebar title="Menu">
                "\u{2630}"
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
