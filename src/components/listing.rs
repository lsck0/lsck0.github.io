#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::models::post::POSTS;

pub struct ListingEntry {
    pub title: &'static str,
    pub description: &'static str,
    pub url: &'static str,
    pub authors: &'static str,
    pub date: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ListingKind {
    Project,
    Publication,
}

#[component]
pub fn Listing(entries: &'static [ListingEntry], kind: ListingKind, empty_message: &'static str) -> impl IntoView {
    if entries.is_empty() {
        return view! { <p class="placeholder">{empty_message}</p> }.into_any();
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
                    let related: Vec<_> = POSTS
                        .iter()
                        .filter(|p| {
                            let value = match kind {
                                ListingKind::Project => p.project(),
                                ListingKind::Publication => p.publication(),
                            };
                            value.is_some_and(|v| v.eq_ignore_ascii_case(entry.title))
                        })
                        .map(|p| (p.slug, p.title()))
                        .collect();
                    let has_related = !related.is_empty();
                    let count = related.len();
                    let related_views = related
                        .into_iter()
                        .enumerate()
                        .map(|(i, (slug, title))| {
                            view! {
                                <A href=format!("/blog/{}", slug)>{title}</A>
                                {(i + 1 < count)
                                    .then(|| {
                                        view! { <span class="related-sep">{"\u{00b7}"}</span> }
                                    })}
                            }
                        })
                        .collect_view();
                    let has_meta = !entry.authors.is_empty() || !entry.date.is_empty();
                    view! {
                        <li>
                            <a href=entry.url target="_blank" class="listing-title">
                                {entry.title}
                            </a>
                            {has_meta
                                .then(|| {
                                    let parts: Vec<&str> = [entry.authors, entry.date]
                                        .into_iter()
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    view! { <p class="listing-meta">{parts.join(" \u{2014} ")}</p> }
                                })}
                            <p class="listing-desc">{entry.description}</p>
                            {has_related
                                .then(|| {
                                    view! {
                                        <div class="related-posts">
                                            <span>"related posts: "</span>
                                            {related_views}
                                        </div>
                                    }
                                })}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
    .into_any();
}
