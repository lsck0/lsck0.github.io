use leptos::prelude::*;
use leptos_router::components::A;

use crate::{components::scramble::ScrambleText, models::post::POSTS};

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

// ============================================================
// Listing Title
// ============================================================

/// Renders a listing title as either a scrambled teaser or a clickable link,
/// depending on whether the title contains scrambled segments.
fn render_listing_title(title_segments: &'static [TextSegment], url: &'static str) -> impl IntoView {
    let is_scrambled = segments_has_scrambled(title_segments);

    if is_scrambled {
        return view! {
            <span class="listing-title teaser">{render_segments(title_segments)}</span>
        }
        .into_any();
    }

    return view! {
        <a href=url target="_blank" class="listing-title">
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
    let matching_posts: Vec<_> = POSTS
        .iter()
        .filter(|post| {
            let value = match kind {
                RelatedKind::Project => post.project(),
                RelatedKind::Publication => post.publication(),
            };
            value.is_some_and(|name| name.eq_ignore_ascii_case(title))
        })
        .map(|post| (post.slug, post.title()))
        .collect();

    if matching_posts.is_empty() {
        return None;
    }

    let total_count = matching_posts.len();
    let post_links = matching_posts
        .into_iter()
        .enumerate()
        .map(|(index, (slug, title))| {
            let is_last = index + 1 >= total_count;
            view! {
                <A href=format!("/blog/{}", slug)>{title}</A>
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

    Effect::new(move |_: Option<()>| {
        let _ = js_sys::eval(
            r#"
            requestAnimationFrame(function() {
                var el = document.getElementById("listing-content");
                if (el && window.renderMathInElement) {
                    renderMathInElement(el, {
                        delimiters: [
                            { left: "\\(", right: "\\)", display: false },
                            { left: "\\[", right: "\\]", display: true }
                        ],
                        throwOnError: false
                    });
                }
            });
            "#,
        );
    });

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
                    let title_view = render_listing_title(entry.title, entry.url);

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
                                })}
                            <p class="listing-desc">{render_segments(entry.description)}</p>
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
    pub url: &'static str,
    pub status: ProjectStatus,
}

#[component]
pub fn ProjectListing(entries: &'static [ProjectEntry]) -> impl IntoView {
    if entries.is_empty() {
        return view! { <p class="placeholder">"No projects yet."</p> }.into_any();
    }

    return view! {
        <div class="listing">
            {ProjectStatus::all()
                .iter()
                .filter_map(|&status| {
                    let group: Vec<_> = entries
                        .iter()
                        .filter(|entry| entry.status == status)
                        .collect();

                    if group.is_empty() {
                        return None;
                    }

                    Some(view! {
                        <div class="listing-category" id=status.id()>
                            <h2 class="listing-category-title">{status.label()}</h2>
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

                                        view! {
                                            <li id=entry_id>
                                                {title_view}
                                                <p class="listing-desc">
                                                    {render_segments(entry.description)}
                                                </p>
                                                {related}
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        </div>
                    })
                })
                .collect_view()}
        </div>
    }
    .into_any();
}
