use std::collections::{BTreeMap, BTreeSet};

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_query_map},
};

use crate::{
    components::{
        layout::Layout,
        storage::{is_bookmarked, is_read, mark_read, mark_unread, set_bookmarked},
    },
    models::{meta::META, post::POSTS, search::fuzzy_score},
    pages::graph::GraphView,
};

const PAGE_SIZE: usize = 10;

// ============================================================
// Types
// ============================================================

#[derive(Clone, Copy, PartialEq)]
enum TagState {
    Neutral,
    Include,
    Exclude,
}

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    List,
    Tree,
    Series,
    Graph,
    Bookmarks,
}

// ============================================================
// Tree helpers
// ============================================================

fn build_folder_tree<'a>(
    posts: &[&'a crate::models::post::Post],
) -> BTreeMap<&'a str, Vec<&'a crate::models::post::Post>> {
    let mut tree: BTreeMap<&str, Vec<&crate::models::post::Post>> = BTreeMap::new();
    for post in posts {
        let folder = if post.folder.is_empty() { "/" } else { post.folder };
        tree.entry(folder).or_default().push(post);
    }
    return tree;
}

// ============================================================
// Component
// ============================================================

#[component]
pub fn BlogPage() -> impl IntoView {
    let (search, set_search) = signal(String::new());
    let (tag_states, set_tag_states) = signal(Vec::<(String, TagState)>::new());
    let (page, set_page) = signal(0usize);
    let (view_mode, set_view_mode) = signal(ViewMode::List);
    let (collapsed_folders, set_collapsed_folders) = signal(BTreeSet::<String>::new());
    let (collapsed_series, set_collapsed_series) = signal(BTreeSet::<String>::new());

    let all_tags: Vec<String> = POSTS
        .iter()
        .flat_map(|post| post.tags())
        .map(|tag| tag.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let query_map = use_query_map();
    let initial_tag = query_map.with_untracked(|map| map.get("tag").map(|tag| tag.to_string()));

    set_tag_states.set(
        all_tags
            .iter()
            .map(|tag| {
                let state = if initial_tag.as_deref() == Some(tag.as_str()) {
                    TagState::Include
                } else {
                    TagState::Neutral
                };
                (tag.clone(), state)
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
        move || !search.get().is_empty() || tag_states.get().iter().any(|(_, state)| *state != TagState::Neutral);

    let filtered_slugs = RwSignal::new(Vec::<usize>::new());

    let filtered_posts = move || -> Vec<&'static crate::models::post::Post> {
        filtered_slugs.get().iter().filter_map(|&i| POSTS.get(i)).collect()
    };

    Effect::new(move |_: Option<()>| {
        let query_text = search.get().to_lowercase();
        let states = tag_states.get();
        let mode = view_mode.get();

        let included: Vec<&str> = states
            .iter()
            .filter(|(_, state)| *state == TagState::Include)
            .map(|(tag, _)| tag.as_str())
            .collect();

        let excluded: Vec<&str> = states
            .iter()
            .filter(|(_, state)| *state == TagState::Exclude)
            .map(|(tag, _)| tag.as_str())
            .collect();

        let tag_filter = |post: &&crate::models::post::Post| -> bool {
            let tags = post.tags();
            let matches_included = included.is_empty() || included.iter().all(|tag| tags.contains(tag));
            let not_excluded = excluded.iter().all(|tag| !tags.contains(tag));
            matches_included && not_excluded
        };

        let bookmark_filter = |post: &&crate::models::post::Post| -> bool {
            if mode != ViewMode::Bookmarks {
                return true;
            }
            is_bookmarked(post.slug)
        };

        let indices: Vec<usize> = if query_text.is_empty() {
            POSTS
                .iter()
                .enumerate()
                .filter(|(_, post)| tag_filter(post) && bookmark_filter(post))
                .map(|(index, _)| index)
                .collect()
        } else {
            let mut scored: Vec<(usize, u32)> = POSTS
                .iter()
                .enumerate()
                .filter(|(_, post)| tag_filter(post) && bookmark_filter(post))
                .filter_map(|(index, post)| {
                    let title_score = fuzzy_score(&query_text, post.title()).map(|score| score.saturating_mul(3));
                    let body_score = fuzzy_score(&query_text, post.body);
                    let block_text = post.labeled_block_text();
                    let block_score = fuzzy_score(&query_text, &block_text).map(|score| score.saturating_mul(2));
                    let best = [title_score, body_score, block_score].into_iter().flatten().max();
                    best.map(|score| (index, score))
                })
                .collect();

            scored.sort_by_key(|entry| std::cmp::Reverse(entry.1));
            if let Some(&(_, best_score)) = scored.first() {
                let threshold = best_score / 3;
                scored.retain(|(_, score)| *score >= threshold);
            }
            scored.into_iter().map(|(index, _)| index).collect()
        };

        filtered_slugs.set(indices);
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

    let view_btn = move |mode: ViewMode, label: &'static str| {
        view! {
            <button
                class=move || {
                    if view_mode.get() == mode { "view-toggle active" } else { "view-toggle" }
                }
                on:click=move |_| set_view_mode.set(mode)
            >
                {label}
            </button>
        }
    };

    return view! {
        <Title text=META.page_title("blog") />
        <Meta
            name="description"
            content=META.page("blog").map(|page| page.description).unwrap_or("")
        />
        <Layout>
            <div class="blog-page">

                // ---- Controls ----
                <div class="blog-toolbar">
                    <div class="blog-search">
                        <input
                            type="text"
                            placeholder="search posts..."
                            prop:value=search
                            on:input=move |event| set_search.set(event_target_value(&event))
                        />
                        <span class="blog-count">
                            {move || {
                                if has_active_filters() {
                                    let total = POSTS.len();
                                    let filtered = filtered_posts().len();
                                    format!("{filtered}/{total}")
                                } else {
                                    String::new()
                                }
                            }}
                        </span>
                    </div>
                    <div class="blog-views">
                        {view_btn(ViewMode::List, "list")} {view_btn(ViewMode::Tree, "tree")}
                        {view_btn(ViewMode::Series, "series")}
                        {view_btn(ViewMode::Bookmarks, "bookmarks")}
                        {view_btn(ViewMode::Graph, "graph")}
                    </div>

                    <div class="blog-filters">
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
                                            "\u{00d7} clear"
                                        </button>
                                    }
                                })
                        }}
                    </div>
                </div>

                // ---- Content area ----
                {move || {
                    match view_mode.get() {
                        ViewMode::List | ViewMode::Bookmarks => {
                            render_list_view(
                                    page_posts,
                                    filtered_posts,
                                    page,
                                    set_page,
                                    total_pages,
                                    view_mode.get(),
                                )
                                .into_any()
                        }
                        ViewMode::Tree => {
                            render_tree_view(
                                    filtered_posts,
                                    collapsed_folders,
                                    set_collapsed_folders,
                                )
                                .into_any()
                        }
                        ViewMode::Series => {
                            render_series_view(
                                    filtered_posts,
                                    collapsed_series,
                                    set_collapsed_series,
                                )
                                .into_any()
                        }
                        ViewMode::Graph => render_graph_view(filtered_posts).into_any(),
                    }
                }}
            </div>
        </Layout>
    };
}

// ============================================================
// List view (also used for bookmarks)
// ============================================================

fn render_list_view(
    page_posts: impl Fn() -> Vec<&'static crate::models::post::Post> + Copy + Send + 'static,
    filtered_posts: impl Fn() -> Vec<&'static crate::models::post::Post> + Copy + Send + 'static,
    page: ReadSignal<usize>,
    set_page: WriteSignal<usize>,
    total_pages: impl Fn() -> usize + Copy + Send + 'static,
    mode: ViewMode,
) -> impl IntoView {
    view! {
        <div class="blog-list-view">
            <ul class="post-list">
                {move || { page_posts().into_iter().map(render_post_card).collect_view() }}
            </ul>

            {move || {
                if filtered_posts().is_empty() {
                    let msg = if mode == ViewMode::Bookmarks {
                        "no bookmarked posts"
                    } else {
                        "no matching posts"
                    };
                    Some(view! { <div class="blog-empty">{msg}</div> }.into_any())
                } else {
                    None
                }
            }}

            {move || {
                if total_pages() > 1 {
                    Some(
                        view! {
                            <div class="pagination">
                                <button
                                    class="page-btn"
                                    disabled=move || page.get() == 0
                                    on:click=move |_| {
                                        set_page
                                            .update(|current| *current = current.saturating_sub(1))
                                    }
                                >
                                    "\u{2190} prev"
                                </button>
                                <span class="page-info">
                                    {move || format!("{}/{}", page.get() + 1, total_pages())}
                                </span>
                                <button
                                    class="page-btn"
                                    disabled=move || total_pages() <= page.get() + 1
                                    on:click=move |_| set_page.update(|current| *current += 1)
                                >
                                    "next \u{2192}"
                                </button>
                            </div>
                        }
                            .into_any(),
                    )
                } else {
                    None
                }
            }}
        </div>
    }
}

// ============================================================
// Tree view
// ============================================================

fn render_tree_view(
    filtered_posts: impl Fn() -> Vec<&'static crate::models::post::Post> + Copy + Send + 'static,
    collapsed_folders: ReadSignal<BTreeSet<String>>,
    set_collapsed_folders: WriteSignal<BTreeSet<String>>,
) -> impl IntoView {
    view! {
        <div class="blog-tree-view">
            {move || {
                let posts = filtered_posts();
                let tree = build_folder_tree(&posts);
                let collapsed = collapsed_folders.get();
                tree.into_iter()
                    .map(|(folder, folder_posts)| {
                        let folder_display = if folder == "/" {
                            "/".to_string()
                        } else {
                            format!("/{folder}/")
                        };
                        let count = folder_posts.len();
                        let is_collapsed = collapsed.contains(folder);
                        let folder_key = folder.to_string();
                        let icon = if is_collapsed { "\u{25b6}" } else { "\u{25bc}" };

                        view! {
                            <div class="tree-section">
                                <button
                                    class="tree-folder-btn"
                                    on:click=move |_| {
                                        let key = folder_key.clone();
                                        set_collapsed_folders
                                            .update(|set| {
                                                if !set.remove(&key) {
                                                    set.insert(key);
                                                }
                                            });
                                    }
                                >
                                    <span class="tree-icon">{icon}</span>
                                    <span class="tree-folder-name">{folder_display.clone()}</span>
                                    <span class="tree-folder-count">{format!("({count})")}</span>
                                </button>
                                {(!is_collapsed)
                                    .then(|| {
                                        let items = folder_posts
                                            .into_iter()
                                            .map(|post| {
                                                let slug = post.slug.to_string();
                                                let (read_sig, set_read_sig) = signal(is_read(post.slug));
                                                let slug_display = post
                                                    .slug
                                                    .rsplit('/')
                                                    .next()
                                                    .unwrap_or(post.slug);
                                                view! {
                                                    <div class="tree-file">
                                                        <A href=post.href() attr:class="tree-file-link">
                                                            {slug_display}
                                                        </A>
                                                        <span class="tree-file-meta">{post.date_formatted()}</span>
                                                        {render_read_badge(slug, read_sig, set_read_sig)}
                                                    </div>
                                                }
                                            })
                                            .collect_view();
                                        view! { <div class="tree-children">{items}</div> }
                                    })}
                            </div>
                        }
                    })
                    .collect_view()
            }}
            {move || {
                (filtered_posts().is_empty())
                    .then(|| view! { <div class="blog-empty">"no matching posts"</div> })
            }}
        </div>
    }
}

// ============================================================
// Series view
// ============================================================

fn render_series_view(
    filtered_posts: impl Fn() -> Vec<&'static crate::models::post::Post> + Copy + Send + 'static,
    collapsed_series: ReadSignal<BTreeSet<String>>,
    set_collapsed_series: WriteSignal<BTreeSet<String>>,
) -> impl IntoView {
    view! {
        <div class="blog-series-view">
            {move || {
                let posts = filtered_posts();
                let mut series_map: BTreeMap<String, Vec<&crate::models::post::Post>> = BTreeMap::new();
                let mut standalone = Vec::new();
                for post in &posts {
                    if let Some(name) = post.series() {
                        series_map.entry(name.to_string()).or_default().push(post);
                    } else {
                        standalone.push(*post);
                    }
                }
                for group in series_map.values_mut() {
                    group.sort_by_key(|post| post.series_order().unwrap_or(0));
                }
                let collapsed = collapsed_series.get();
                let series_views = series_map
                    .into_iter()
                    .map(|(name, series_posts)| {
                        let total = series_posts.len();
                        let read_count = series_posts
                            .iter()
                            .filter(|post| is_read(post.slug))
                            .count();
                        let is_collapsed = collapsed.contains(&name);
                        let name_key = name.clone();
                        let icon = if is_collapsed { "\u{25b6}" } else { "\u{25bc}" };

                        view! {
                            <div class="series-section">
                                <button
                                    class="series-header-btn"
                                    on:click=move |_| {
                                        let key = name_key.clone();
                                        set_collapsed_series
                                            .update(|set| {
                                                if !set.remove(&key) {
                                                    set.insert(key);
                                                }
                                            });
                                    }
                                >
                                    <span class="tree-icon">{icon}</span>
                                    <span class="series-name">{name.clone()}</span>
                                    <span class="series-progress">
                                        {format!("{read_count}/{total}")}
                                    </span>
                                </button>
                                {(!is_collapsed)
                                    .then(|| {
                                        let items = series_posts
                                            .into_iter()
                                            .enumerate()
                                            .map(|(i, post)| {
                                                let slug = post.slug.to_string();
                                                let (read_sig, set_read_sig) = signal(is_read(post.slug));
                                                view! {
                                                    <div class="series-entry">
                                                        <span class="series-num">{format!("{}.", i + 1)}</span>
                                                        <A href=post.href() attr:class="series-link">
                                                            {post.title()}
                                                        </A>
                                                        {render_read_badge(slug, read_sig, set_read_sig)}
                                                    </div>
                                                }
                                            })
                                            .collect_view();
                                        view! { <div class="series-entries">{items}</div> }
                                    })}
                            </div>
                        }
                    })
                    .collect_view();
                let standalone_view = (!standalone.is_empty())
                    .then(|| {
                        let total = standalone.len();
                        let read_count = standalone
                            .iter()
                            .filter(|post| is_read(post.slug))
                            .count();
                        let is_collapsed = collapsed.contains("__standalone__");
                        let icon = if is_collapsed { "\u{25b6}" } else { "\u{25bc}" };
                        let items = standalone
                            .into_iter()
                            .map(|post| {
                                let slug = post.slug.to_string();
                                let (read_sig, set_read_sig) = signal(is_read(post.slug));
                                view! {
                                    <div class="series-entry">
                                        <A href=post.href() attr:class="series-link">
                                            {post.title()}
                                        </A>
                                        <span class="tree-file-meta">{post.date_formatted()}</span>
                                        {render_read_badge(slug, read_sig, set_read_sig)}
                                    </div>
                                }
                            })
                            .collect_view();
                        view! {
                            <div class="series-section standalone-section">
                                <button
                                    class="series-header-btn"
                                    on:click=move |_| {
                                        let key = "__standalone__".to_string();
                                        set_collapsed_series
                                            .update(|set| {
                                                if !set.remove(&key) {
                                                    set.insert(key);
                                                }
                                            });
                                    }
                                >
                                    <span class="tree-icon">{icon}</span>
                                    <span class="series-name">"Standalone"</span>
                                    <span class="series-progress">
                                        {format!("{read_count}/{total}")}
                                    </span>
                                </button>
                                {(!is_collapsed)
                                    .then(|| {
                                        view! { <div class="series-entries">{items}</div> }
                                    })}
                            </div>
                        }
                    });

                view! {
                    {series_views}
                    {standalone_view}
                }
            }}
        </div>
    }
}

// ============================================================
// Graph view
// ============================================================

fn render_graph_view(
    filtered_posts: impl Fn() -> Vec<&'static crate::models::post::Post> + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let visible_slugs = Signal::derive(move || {
        filtered_posts()
            .iter()
            .map(|post| post.slug.to_string())
            .collect::<Vec<_>>()
    });
    view! {
        <div class="blog-graph-view">
            <GraphView visible_slugs=visible_slugs />
        </div>
    }
}

// ============================================================
// Post card (list view)
// ============================================================

fn render_post_card(post: &crate::models::post::Post) -> impl IntoView {
    let slug = post.slug.to_string();
    let slug_for_read = slug.clone();
    let href = post.href();
    let title = post.title().to_string();
    let date = post.date_formatted();
    let (read_sig, set_read_sig) = signal(is_read(post.slug));
    let (bookmarked, set_bookmarked_sig) = signal(is_bookmarked(post.slug));

    let series_hint = post.series().map(|name| {
        let series_posts = post.series_posts();
        let series_idx = series_posts
            .iter()
            .position(|series_post| series_post.slug == post.slug)
            .map(|index| index + 1)
            .unwrap_or(0);
        let total = series_posts.len();
        format!("[{name} {series_idx}/{total}]")
    });

    let tags_str = post
        .tags()
        .into_iter()
        .map(|tag| format!("#{tag}"))
        .collect::<Vec<_>>()
        .join("  ");
    let description = post.description().to_string();
    let has_description = !description.is_empty();

    let toggle_bookmark = move |_| {
        let next = !bookmarked.get();
        set_bookmarked(&slug, next);
        set_bookmarked_sig.set(next);
    };

    return view! {
        <li class="post-card">
            <div class="post-card-header">
                <button
                    class=move || {
                        if bookmarked.get() {
                            "post-bookmark-btn active"
                        } else {
                            "post-bookmark-btn"
                        }
                    }
                    on:click=toggle_bookmark
                    title="toggle bookmark"
                ></button>
                <A href=href attr:class="post-title-link">
                    {title}
                </A>
                {series_hint.map(|hint| view! { <span class="post-series">{hint}</span> })}
                {render_read_badge(slug_for_read, read_sig, set_read_sig)}
            </div>
            <div class="post-card-meta">
                <span class="post-date">{date}</span>
                <span class="post-tags">{tags_str}</span>
            </div>
            {has_description
                .then(|| {
                    view! { <div class="post-card-desc">{description}</div> }
                })}
        </li>
    };
}

// ============================================================
// Read badge (shared clickable component)
// ============================================================

fn render_read_badge(slug: String, read_sig: ReadSignal<bool>, set_read_sig: WriteSignal<bool>) -> impl IntoView {
    let toggle_read = move |_: leptos::ev::MouseEvent| {
        let current = read_sig.get();
        if current {
            mark_unread(&slug);
        } else {
            mark_read(&slug);
        }
        set_read_sig.set(!current);
    };

    view! {
        <button
            class=move || if read_sig.get() { "read-badge read" } else { "read-badge unread" }
            on:click=toggle_read
            title=move || {
                if read_sig.get() { "click to mark as unread" } else { "click to mark as read" }
            }
        >
            {move || if read_sig.get() { "read" } else { "unread" }}
        </button>
    }
}
