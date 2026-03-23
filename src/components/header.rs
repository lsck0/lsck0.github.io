use leptos::prelude::*;
use leptos_router::components::A;

use super::SidebarState;

#[component]
pub fn Header() -> impl IntoView {
    let sidebar_state = use_context::<SidebarState>();

    let toggle_sidebar = move |_| {
        if let Some(state) = sidebar_state {
            // On mobile, toggle the mobile overlay
            // On desktop, toggle the desktop sidebar
            let window = web_sys::window().unwrap();
            let width = window.inner_width().unwrap().as_f64().unwrap_or(0.0);
            if width <= 768.0 {
                state.is_mobile_open.update(|is_open| *is_open = !*is_open);
            } else {
                state.is_desktop_open.update(|is_open| *is_open = !*is_open);
            }
        }
    };

    let is_open = move || {
        if let Some(state) = sidebar_state {
            state.is_mobile_open.get() || state.is_desktop_open.get()
        } else {
            false
        }
    };

    return view! {
        <header class="site-header">
            <button class="sidebar-toggle" on:click=toggle_sidebar title="Menu">
                {move || if is_open() { "\u{2715}" } else { "\u{2630}" }}
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
