use leptos::prelude::*;
use leptos_meta::*;
use macros::include_projects;

use crate::{
    components::{
        layout::Layout,
        listing::{ProjectEntry, ProjectListing, ProjectStatus, TextSegment},
    },
    models::meta::META,
};

pub const PROJECTS: &[ProjectEntry] = include_projects!();

#[component]
pub fn ProjectsPage() -> impl IntoView {
    return view! {
        <Title text=META.page_title("projects") />
        <Meta
            name="description"
            content=META.page("projects").map(|page| page.description).unwrap_or("")
        />
        <Layout>
            <div class="listing-page">
                <ProjectListing entries=PROJECTS />
            </div>
        </Layout>
    };
}
