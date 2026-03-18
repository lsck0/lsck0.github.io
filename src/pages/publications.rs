use leptos::prelude::*;
use leptos_meta::*;
use macros::include_publications;

use crate::{
    components::{
        layout::Layout,
        listing::{PublicationEntry, PublicationListing, TextSegment},
    },
    models::meta::META,
};

pub const PUBLICATIONS: &[PublicationEntry] = include_publications!();

#[component]
pub fn PublicationsPage() -> impl IntoView {
    return view! {
        <Title text=META.page_title("publications") />
        <Meta
            name="description"
            content=META.page("publications").map(|page| page.description).unwrap_or("")
        />
        <Layout>
            <div class="listing-page">
                <h1>"Publications"</h1>
                <PublicationListing entries=PUBLICATIONS />
            </div>
        </Layout>
    };
}
