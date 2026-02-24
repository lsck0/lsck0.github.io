#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_router::{components::*, path};

pub mod components;
pub mod pages;

use crate::pages::index::Index;

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        return view! {
            <ErrorBoundary fallback=|errors| {
                view! {
                    <h1>"Something went wrong."</h1>

                    <p>"Errors: "</p>
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
                    <Routes fallback=|| view! { NotFound }>
                        <Route path=path!("/") view=Index />
                    </Routes>
                </Router>

            </ErrorBoundary>
        };
    });
}
