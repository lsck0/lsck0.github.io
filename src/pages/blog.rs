#![allow(clippy::needless_return)]

use std::collections::BTreeSet;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_query_map},
};

use crate::{components::layout::Layout, models::post::POSTS};

const PAGE_SIZE: usize = 10;

#[derive(Clone, Copy, PartialEq)]
enum TagState {
    Neutral,
    Include,
    Exclude,
}

#[component]
pub fn Blog() -> impl IntoView {
    let (search, set_search) = signal(String::new());
    let (tag_states, set_tag_states) = signal(Vec::<(String, TagState)>::new());
    let (page, set_page) = signal(0usize);

    let all_tags: Vec<String> = POSTS
        .iter()
        .flat_map(|p| p.tags())
        .map(|t| t.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let query = use_query_map();
    let initial_tag = query.with_untracked(|map| map.get("tag").map(|s| s.to_string()));

    set_tag_states.set(
        all_tags
            .iter()
            .map(|t| {
                let state = if initial_tag.as_deref() == Some(t.as_str()) {
                    TagState::Include
                } else {
                    TagState::Neutral
                };
                (t.clone(), state)
            })
            .collect(),
    );

    if initial_tag.is_some() {
        let navigate = use_navigate();
        request_animation_frame(move || {
            navigate("/blog", Default::default());
        });
    }

    let reset_filters = move |_| {
        set_search.set(String::new());
        set_tag_states.update(|states| {
            for entry in states.iter_mut() {
                entry.1 = TagState::Neutral;
            }
        });
    };

    let has_active_filters =
        move || !search.get().is_empty() || tag_states.get().iter().any(|(_, s)| *s != TagState::Neutral);

    let filtered_posts = move || {
        let q = search.get().to_lowercase();
        let states = tag_states.get();

        let included: Vec<&str> = states
            .iter()
            .filter(|(_, s)| *s == TagState::Include)
            .map(|(t, _)| t.as_str())
            .collect();

        let excluded: Vec<&str> = states
            .iter()
            .filter(|(_, s)| *s == TagState::Exclude)
            .map(|(t, _)| t.as_str())
            .collect();

        POSTS
            .iter()
            .filter(|post| {
                let post_tags = post.tags();
                let include_match = included.is_empty() || included.iter().all(|t| post_tags.contains(t));
                let exclude_match = excluded.iter().all(|t| !post_tags.contains(t));
                let search_match = q.is_empty()
                    || post.title().to_lowercase().contains(&q)
                    || post.content.to_lowercase().contains(&q);
                include_match && exclude_match && search_match
            })
            .collect::<Vec<_>>()
    };

    // Reset to page 0 whenever filters change
    Effect::new(move |_: Option<()>| {
        let _ = search.get();
        let _ = tag_states.get();
        set_page.set(0);
    });

    let total_pages = move || {
        let count = filtered_posts().len();
        if count == 0 { 1 } else { count.div_ceil(PAGE_SIZE) }
    };

    let page_posts = move || {
        let all = filtered_posts();
        let start = page.get() * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(all.len());
        if start >= all.len() {
            vec![]
        } else {
            all[start..end].to_vec()
        }
    };

    return view! {
        <Title text="\u{03bb} lsck0 \u{2014} blog" />
        <Layout>
            <div class="controls">
                <input
                    type="text"
                    placeholder="search posts..."
                    prop:value=search
                    on:input=move |e| set_search.set(event_target_value(&e))
                />
                <div class="tag-filters">
                    {move || {
                        tag_states
                            .get()
                            .into_iter()
                            .map(|(tag, state)| {
                                let tag_for_click = tag.clone();
                                let class = match state {
                                    TagState::Neutral => "tag",
                                    TagState::Include => "tag include",
                                    TagState::Exclude => "tag exclude",
                                };
                                return view! {
                                    <button
                                        class=class
                                        on:click=move |_| {
                                            let tag = tag_for_click.clone();
                                            set_tag_states
                                                .update(|states| {
                                                    if let Some(entry) = states
                                                        .iter_mut()
                                                        .find(|(t, _)| *t == tag)
                                                    {
                                                        entry.1 = match entry.1 {
                                                            TagState::Neutral => TagState::Include,
                                                            TagState::Include => TagState::Exclude,
                                                            TagState::Exclude => TagState::Neutral,
                                                        };
                                                    }
                                                });
                                        }
                                    >
                                        {format!("#{tag}")}
                                    </button>
                                };
                            })
                            .collect_view()
                    }}
                    {move || {
                        has_active_filters()
                            .then(|| {
                                view! {
                                    <button class="tag reset" on:click=reset_filters>
                                        "reset"
                                    </button>
                                }
                            })
                    }}
                </div>
            </div>

            <ul class="post-list">
                {move || {
                    page_posts()
                        .into_iter()
                        .map(|post| {
                            return view! {
                                <li>
                                    <A
                                        href=format!("/blog/{}", post.slug)
                                        attr:class="post-title-link"
                                    >
                                        {post.title()}
                                    </A>
                                    <div class="post-info">
                                        <span class="date">{post.date_formatted()}</span>
                                        <span class="post-tags">
                                            {post
                                                .tags()
                                                .into_iter()
                                                .map(|t| {
                                                    view! { <span class="tag">{format!("#{t}")}</span> }
                                                })
                                                .collect_view()}
                                        </span>
                                    </div>
                                    {(!post.description().is_empty())
                                        .then(|| {
                                            view! { <p class="description">{post.description()}</p> }
                                        })}
                                </li>
                            };
                        })
                        .collect_view()
                }}
            </ul>

            {move || {
                (total_pages() > 1)
                    .then(|| {
                        view! {
                            <div class="pagination">
                                <button
                                    class="page-btn"
                                    disabled=move || page.get() == 0
                                    on:click=move |_| set_page.update(|p| *p = p.saturating_sub(1))
                                >
                                    "prev"
                                </button>
                                <span class="page-info">
                                    {move || format!("{} / {}", page.get() + 1, total_pages())}
                                </span>
                                <button
                                    class="page-btn"
                                    disabled=move || total_pages() <= page.get() + 1
                                    on:click=move |_| set_page.update(|p| *p += 1)
                                >
                                    "next"
                                </button>
                            </div>
                        }
                    })
            }}
        </Layout>
    };
}
