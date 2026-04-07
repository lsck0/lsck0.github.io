#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, hooks::use_location, path};

pub mod components;
pub mod models;
pub mod pages;

use crate::{
    components::SidebarState,
    pages::{
        about::AboutPage, blog::BlogPage, home::HomePage, imprint::ImprintPage, not_found::NotFoundPage,
        post::PostPage, privacy::PrivacyPage, projects::ProjectsPage, publications::PublicationsPage, tos::TosPage,
    },
};

// ============================================================
// View Transitions
// ============================================================

/// Triggers the View Transitions API on route changes for smooth page transitions.
/// Falls back silently on browsers that don't support `document.startViewTransition()`.
#[component]
fn ViewTransitionTrigger() -> impl IntoView {
    let location = use_location();
    let previous_path = RwSignal::new(String::new());

    Effect::new(move |_: Option<()>| {
        let current_path = location.pathname.get();
        let old_path = previous_path.get_untracked();
        if !old_path.is_empty() && old_path != current_path {
            let _ = js_sys::eval("document.startViewTransition && document.startViewTransition(function(){})");
        }
        previous_path.set(current_path);
    });

    return;
}

// ============================================================
// Application Entry Point
// ============================================================

fn main() {
    _ = console_log::init_with_level(log::Level::Warn);
    console_error_panic_hook::set_once();

    // Inject KaTeX macros from content/macros.tex into JS global scope
    let macros_js = format!("window.KATEX_MACROS={};", models::post::KATEX_MACROS_JSON);
    let _ = js_sys::eval(&macros_js);

    mount_to_body(|| {
        provide_meta_context();
        provide_context(SidebarState::new());

        return view! {
            <ErrorBoundary fallback=|errors| {
                view! {
                    <h1>"Something went wrong."</h1>
                    <ul>
                        {move || {
                            errors
                                .get()
                                .into_iter()
                                .map(|(_, error)| view! { <li>{error.to_string()}</li> })
                                .collect_view()
                        }}
                    </ul>
                }
            }>

                <Router>
                    <ViewTransitionTrigger />
                    <Routes fallback=|| view! { <NotFoundPage /> }>
                        <Route path=path!("/") view=HomePage />
                        <Route path=path!("/about") view=AboutPage />
                        <Route path=path!("/blog") view=BlogPage />
                        <Route path=path!("/blog/*slug") view=PostPage />
                        <Route path=path!("/projects") view=ProjectsPage />
                        <Route path=path!("/publications") view=PublicationsPage />
                        <Route path=path!("/imprint") view=ImprintPage />
                        <Route path=path!("/privacy") view=PrivacyPage />
                        <Route path=path!("/tos") view=TosPage />
                    </Routes>
                </Router>
            </ErrorBoundary>
        };
    });
}
