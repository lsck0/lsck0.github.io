#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

pub mod components;
pub mod models;
pub mod pages;

use crate::{
    components::SidebarState,
    pages::{
        about::About, blog::Blog, home::Home, imprint::Imprint, not_found::NotFound, post::PostPage, privacy::Privacy,
        projects::Projects, publications::Publications,
    },
};

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

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
                                .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                .collect_view()
                        }}
                    </ul>
                }
            }>

                <Router>
                    <Routes fallback=|| view! { <NotFound /> }>
                        <Route path=path!("/") view=Home />
                        <Route path=path!("/about") view=About />
                        <Route path=path!("/blog") view=Blog />
                        <Route path=path!("/blog/*slug") view=PostPage />
                        <Route path=path!("/projects") view=Projects />
                        <Route path=path!("/publications") view=Publications />
                        <Route path=path!("/imprint") view=Imprint />
                        <Route path=path!("/privacy") view=Privacy />
                    </Routes>
                </Router>

            </ErrorBoundary>
        };
    });
}
