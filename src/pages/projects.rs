#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_meta::*;

use crate::components::{
    layout::Layout,
    listing::{Listing, ListingEntry, ListingKind},
};

const PROJECTS: &[ListingEntry] = &[
    ListingEntry {
        title: "lsck0.github.io",
        description: "This site — a personal blog and portfolio built with Rust, Leptos, and WebAssembly.",
        url: "https://github.com/lsck0/lsck0.github.io",
        authors: "",
        date: "",
    },
    ListingEntry {
        title: "plt",
        description: "Notes and experiments on programming language theory, type systems, and formal methods.",
        url: "https://github.com/lsck0",
        authors: "",
        date: "",
    },
];

#[component]
pub fn Projects() -> impl IntoView {
    return view! {
        <Title text="\u{03bb} lsck0 \u{2014} projects" />
        <Layout>
            <div class="listing-page">
                <h1>"Projects"</h1>
                <Listing
                    entries=PROJECTS
                    kind=ListingKind::Project
                    empty_message="No projects yet."
                />
            </div>
        </Layout>
    };
}
