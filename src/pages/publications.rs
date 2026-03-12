#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_meta::*;

use crate::components::{
    layout::Layout,
    listing::{Listing, ListingEntry, ListingKind},
};

const PUBLICATIONS: &[ListingEntry] = &[
    ListingEntry {
        title: "cs",
        description: "Sample entry — replace with actual publication details, venue, and year.",
        url: "https://arxiv.org",
        authors: "L. Sandrock, A. Coauthor",
        date: "2026",
    },
    ListingEntry {
        title: "math",
        description: "On the structure of \\(\\mathbb{Z}[i]\\) as a Euclidean domain — sample entry.",
        url: "https://arxiv.org",
        authors: "L. Sandrock, B. Coauthor",
        date: "2025",
    },
];

#[component]
pub fn Publications() -> impl IntoView {
    return view! {
        <Title text="\u{03bb} lsck0 \u{2014} publications" />
        <Layout>
            <div class="listing-page">
                <h1>"Publications"</h1>
                <Listing
                    entries=PUBLICATIONS
                    kind=ListingKind::Publication
                    empty_message="No publications yet."
                />
            </div>
        </Layout>
    };
}
