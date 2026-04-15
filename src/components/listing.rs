use leptos::prelude::*;
use leptos_router::components::A;

use crate::{
    components::{render::call_render_post, scramble::ScrambleText},
    models::post::{POSTS, Post},
};

// ============================================================
// Text Segments
// ============================================================

#[derive(Clone, Copy)]
pub enum TextSegment {
    Text(&'static str),
    Scrambled(usize),
}

pub fn segments_empty(segments: &[TextSegment]) -> bool {
    segments.iter().all(|segment| match segment {
        TextSegment::Text(text) => text.is_empty(),
        TextSegment::Scrambled(length) => *length == 0,
    })
}

pub fn segments_has_scrambled(segments: &[TextSegment]) -> bool {
    segments
        .iter()
        .any(|segment| matches!(segment, TextSegment::Scrambled(_)))
}

pub fn segments_plain_value(segments: &[TextSegment]) -> String {
    segments
        .iter()
        .filter_map(|segment| match segment {
            TextSegment::Text(text) => Some(*text),
            TextSegment::Scrambled(_) => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

pub fn render_segments(segments: &'static [TextSegment]) -> impl IntoView {
    segments
        .iter()
        .map(|segment| match segment {
            TextSegment::Text(text) => view! { <span>{*text}</span> }.into_any(),
            TextSegment::Scrambled(length) => view! { <ScrambleText len=*length /> }.into_any(),
        })
        .collect_view()
}

/// Renders a sidebar label: scrambled titles get a teaser wrapper, plain titles render as text.
pub fn sidebar_label(title: &'static [TextSegment]) -> impl IntoView {
    if segments_has_scrambled(title) {
        view! { <span class="teaser">{render_segments(title)}</span> }.into_any()
    } else {
        view! { {segments_plain_value(title)} }.into_any()
    }
}

// ============================================================
// Tools
// ============================================================

fn render_tools(tools: &'static [&'static str]) -> impl IntoView {
    if tools.is_empty() {
        return None;
    }
    Some(view! {
        <div class="project-tools">
            {tools
                .iter()
                .map(|tool| view! { <span class="project-tool">{*tool}</span> })
                .collect_view()}
        </div>
    })
}

// ============================================================
// Listing Title
// ============================================================

/// Renders a listing title as either a scrambled teaser or a clickable link,
/// depending on whether the title contains scrambled segments or url is present.
fn render_listing_title(title_segments: &'static [TextSegment], url: Option<&'static str>) -> impl IntoView {
    let is_scrambled = segments_has_scrambled(title_segments);

    if is_scrambled || url.is_none() {
        return view! { <span class="listing-title">{render_segments(title_segments)}</span> }.into_any();
    }

    return view! {
        <a href=url.unwrap() target="_blank" class="listing-title">
            {render_segments(title_segments)}
        </a>
    }
    .into_any();
}

// ============================================================
// Related Posts
// ============================================================

#[derive(Clone, Copy)]
pub enum RelatedKind {
    Project,
    Publication,
}

pub fn render_related_posts(title: &str, kind: RelatedKind) -> Option<impl IntoView + use<>> {
    let matching_posts: Vec<&'static Post> = POSTS
        .iter()
        .filter(|post| {
            let value = match kind {
                RelatedKind::Project => post.project(),
                RelatedKind::Publication => post.publication(),
            };
            value.is_some_and(|name| name.eq_ignore_ascii_case(title))
        })
        .collect();

    if matching_posts.is_empty() {
        return None;
    }

    let total_count = matching_posts.len();
    let post_links = matching_posts
        .into_iter()
        .enumerate()
        .map(|(index, post)| {
            let is_last = index + 1 >= total_count;
            let slug = post.slug();
            let title = post.title();
            let desc = post.description();
            let tags = post.tags().join(", ");
            let series = post.series().unwrap_or("");

            view! {
                <A
                    href=format!("/blog/{}", slug)
                    attr:data-post-title=title
                    attr:data-post-desc=desc
                    attr:data-post-tags=tags
                    attr:data-post-series=series
                >
                    {title}
                </A>
                {(!is_last).then(|| view! { <span class="related-sep">{"\u{00b7}"}</span> })}
            }
        })
        .collect_view();

    return Some(view! {
        <div class="related-posts">
            <span>"related posts: "</span>
            {post_links}
        </div>
    });
}

// ============================================================
// Publication Listing
// ============================================================

pub struct PublicationEntry {
    pub title: &'static [TextSegment],
    pub description: &'static [TextSegment],
    pub url: &'static str,
    pub authors: &'static [TextSegment],
    pub date: &'static [TextSegment],
}

#[component]
pub fn PublicationListing(entries: &'static [PublicationEntry]) -> impl IntoView {
    if entries.is_empty() {
        return view! { <p class="placeholder">"No publications yet."</p> }.into_any();
    }

    call_render_post();

    return view! {
        <ul id="listing-content" class="listing">
            {entries
                .iter()
                .map(|entry| {
                    let plain_title = segments_plain_value(entry.title);
                    let entry_id = plain_title.clone();
                    let has_authors = !segments_empty(entry.authors);
                    let has_date = !segments_empty(entry.date);
                    let has_metadata = has_authors || has_date;
                    let related = render_related_posts(&plain_title, RelatedKind::Publication);
                    let pub_url = if entry.url.is_empty() { None } else { Some(entry.url) };
                    let title_view = render_listing_title(entry.title, pub_url);

                    view! {
                        <li id=entry_id>
                            {title_view}
                            {has_metadata
                                .then(|| {
                                    view! {
                                        <p class="listing-meta">
                                            {render_segments(entry.authors)}
                                            {(has_authors && has_date).then_some(" \u{2014} ")}
                                            {render_segments(entry.date)}
                                        </p>
                                    }
                                })} <p class="listing-desc">{render_segments(entry.description)}</p>
                            {related}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
    .into_any();
}

// ============================================================
// Project Listing
// ============================================================

#[derive(Clone, Copy, PartialEq)]
pub enum ProjectStatus {
    Maintained,
    WorkInProgress,
    Planned,
    Abandoned,
}

impl ProjectStatus {
    pub fn label(self) -> &'static str {
        match self {
            ProjectStatus::Maintained => "maintained",
            ProjectStatus::WorkInProgress => "work in progress",
            ProjectStatus::Planned => "planned",
            ProjectStatus::Abandoned => "abandoned",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            ProjectStatus::Maintained => "maintained",
            ProjectStatus::WorkInProgress => "wip",
            ProjectStatus::Planned => "planned",
            ProjectStatus::Abandoned => "abandoned",
        }
    }

    pub fn all() -> &'static [ProjectStatus] {
        &[
            ProjectStatus::Maintained,
            ProjectStatus::WorkInProgress,
            ProjectStatus::Planned,
            ProjectStatus::Abandoned,
        ]
    }
}

pub struct ProjectEntry {
    pub title: &'static [TextSegment],
    pub description: &'static [TextSegment],
    pub url: Option<&'static str>,
    pub status: ProjectStatus,
    pub company: Option<&'static str>,
    pub anonymous: bool,
    pub tools: &'static [&'static str],
}

impl ProjectEntry {
    pub fn is_professional(&self) -> bool {
        self.company.is_some() || self.anonymous
    }
}

#[component]
pub fn ProjectListing(entries: &'static [ProjectEntry]) -> impl IntoView {
    if entries.is_empty() {
        return view! { <p class="placeholder">"No projects yet."</p> }.into_any();
    }

    call_render_post();

    let professional: Vec<_> = entries.iter().filter(|e| e.is_professional()).collect();
    let private: Vec<_> = entries.iter().filter(|e| !e.is_professional()).collect();
    let has_professional = !professional.is_empty();
    let has_private = !private.is_empty();

    let professional_views: Vec<_> = professional
        .into_iter()
        .map(|entry| {
            let plain_title = segments_plain_value(entry.title);
            let entry_id = plain_title.clone();
            let title_view = render_listing_title(entry.title, entry.url);
            let company_indicator = entry
                .company
                .map(|company| view! { <span class="project-company">{" \u{00b7} "}{company}</span> }.into_any())
                .unwrap_or_else(|| {
                    if entry.anonymous {
                        view! { <span class="project-company">{" \u{00b7} "}"Professional"</span> }.into_any()
                    } else {
                        view! { <span /> }.into_any()
                    }
                });
            let tools_view = render_tools(entry.tools);

            view! {
                <div class="professional-project" id=entry_id>
                    <div class="professional-project-header">{title_view} {company_indicator}</div>
                    <p class="listing-desc">{render_segments(entry.description)}</p>
                    {tools_view}
                </div>
            }
        })
        .collect();

    let private_status_views: Vec<_> = ProjectStatus::all()
        .iter()
        .filter_map(|&status| {
            let group: Vec<_> = private.iter().filter(|entry| entry.status == status).copied().collect();
            if group.is_empty() {
                return None;
            }
            let id = format!("private-{}", status.id());
            Some(view! {
                <div class="listing-category" id=id>
                    <h3 class="listing-category-title">{status.label()}</h3>
                    <ul>
                        {group
                            .into_iter()
                            .map(|entry| {
                                let plain_title = segments_plain_value(entry.title);
                                let entry_id = plain_title.clone();
                                let related = render_related_posts(
                                    &plain_title,
                                    RelatedKind::Project,
                                );
                                let title_view = render_listing_title(entry.title, entry.url);
                                let tools_view = render_tools(entry.tools);

                                view! {
                                    <li id=entry_id>
                                        {title_view}
                                        <p class="listing-desc">
                                            {render_segments(entry.description)}
                                        </p> {tools_view} {related}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </div>
            })
        })
        .collect();

    return view! {
        <div id="listing-content" class="listing">
            {if has_private {
                view! {
                    <div class="project-category">
                        <h1 class="project-category-title">"Private Projects"</h1>
                        <div class="private-projects-list">{private_status_views}</div>
                    </div>
                }
                    .into_any()
            } else {
                view! { <div /> }.into_any()
            }}
            {if has_professional {
                view! {
                    <div class="project-category">
                        <h1 class="project-category-title">"Professional Projects"</h1>
                        <div class="professional-projects-list">{professional_views}</div>
                    </div>
                }
                    .into_any()
            } else {
                view! { <div /> }.into_any()
            }}
        </div>
    }
    .into_any();
}
