use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::A, hooks::use_params_map};
use wasm_bindgen::{JsCast, closure::Closure};

use crate::{
    components::{
        SidebarState,
        layout::Layout,
        post_search::PostSearch,
        render::markdown_to_html,
        storage::{is_bookmarked, set_bookmarked},
    },
    models::{
        meta::META,
        post::{POSTS, Post},
    },
    pages::not_found::NotFoundPage,
};

// ============================================================
// Reference link data
// ============================================================

struct PostLink {
    url: String,
    title: String,
    description: String,
    tags: String,
    series: String,
    anchors: Vec<String>,
}

impl PostLink {
    fn from_post(post: &Post, url: &str, anchors: Vec<String>) -> Self {
        return Self {
            url: url.to_string(),
            title: post.title().to_string(),
            description: post.description().to_string(),
            tags: post.tags().join(", "),
            series: post.series().unwrap_or("").to_string(),
            anchors,
        };
    }
}

struct ExternalLink {
    url: String,
    anchors: Vec<String>,
}

// ============================================================
// Component
// ============================================================

#[component]
pub fn PostPage() -> impl IntoView {
    let params = use_params_map();
    let scroll_progress = RwSignal::new(0.0f64);
    let show_scroll_top = RwSignal::new(false);

    let current_post = move || {
        params.with(|map| {
            let slug = map.get("slug")?;
            let slug = slug.trim_end_matches('/');
            POSTS.iter().find(|post| post.slug == slug)
        })
    };

    setup_sidebar_expansion(current_post);
    setup_scroll_progress(scroll_progress, show_scroll_top);

    Effect::new(move |_: Option<()>| {
        if let Some(post) = current_post() {
            crate::components::storage::mark_read(post.slug);
        }
    });

    return view! {
        {move || match current_post() {
            None => view! { <NotFoundPage /> }.into_any(),
            Some(post) => render_post(post, scroll_progress, show_scroll_top).into_any(),
        }}
    };
}

// ============================================================
// Sidebar expansion
// ============================================================

fn setup_sidebar_expansion(current_post: impl Fn() -> Option<&'static Post> + Send + Sync + 'static) {
    Effect::new(move |_: Option<()>| {
        if let Some(post) = current_post() {
            if let Some(state) = use_context::<SidebarState>() {
                state.is_blog_open.set(true);
                if !post.folder.is_empty() {
                    state.collapsed_folders.update(|folders| {
                        let mut folder_prefix = String::new();
                        for (segment_index, segment) in post.folder.split('/').enumerate() {
                            if segment_index > 0 {
                                folder_prefix.push('/');
                            }
                            folder_prefix.push_str(segment);
                            folders.remove(&folder_prefix);
                        }
                    });
                }
            }
            let _ = js_sys::eval("renderPost()");
        }
    });
}

// ============================================================
// Scroll progress + scroll-to-top
// ============================================================

fn setup_scroll_progress(scroll_progress: RwSignal<f64>, show_scroll_top: RwSignal<bool>) {
    Effect::new(move |_: Option<()>| {
        let window = web_sys::window().unwrap();
        let closure = Closure::<dyn Fn()>::new(move || {
            let Some(window) = web_sys::window() else { return };
            let Some(document) = window.document() else { return };
            let Some(post_content) = document.get_element_by_id("post-content") else {
                return;
            };
            let bounding_rect = post_content.get_bounding_client_rect();
            let viewport_height = window.inner_height().unwrap().as_f64().unwrap_or(1.0);

            let scrollable_distance = bounding_rect.height() - viewport_height;
            if scrollable_distance <= 0.0 {
                scroll_progress.set(-1.0);
                show_scroll_top.set(false);
                return;
            }
            let progress_percent = (-bounding_rect.top() / scrollable_distance * 100.0).clamp(0.0, 100.0);
            scroll_progress.set(progress_percent);
            show_scroll_top.set(-bounding_rect.top() > viewport_height * 0.5);
        });

        let handler: js_sys::Function = closure.into_js_value().unchecked_into();
        window.add_event_listener_with_callback("scroll", &handler).unwrap();

        on_cleanup(move || {
            if let Some(window) = web_sys::window() {
                let _ = window.remove_event_listener_with_callback("scroll", &handler);
            }
        });
    });
}

// ============================================================
// Post rendering
// ============================================================

fn render_post(post: &'static Post, scroll_progress: RwSignal<f64>, show_scroll_top: RwSignal<bool>) -> impl IntoView {
    let rendered = markdown_to_html(post.body);
    let content_html = rendered.html;
    let link_occurrences = rendered.link_occurrences;

    let outgoing_links: Vec<PostLink> = post
        .outgoing_links()
        .iter()
        .map(|linked_post| {
            let url = linked_post.href();
            let anchors = link_occurrences.get(&url).cloned().unwrap_or_default();
            PostLink::from_post(linked_post, &url, anchors)
        })
        .collect();

    let incoming_links: Vec<PostLink> = post
        .incoming_links()
        .iter()
        .map(|linked_post| {
            let url = linked_post.href();
            PostLink::from_post(linked_post, &url, vec![])
        })
        .collect();

    let external_links: Vec<ExternalLink> = post
        .external_links
        .iter()
        .map(|url| {
            let url_without_fragment = url.split('#').next().unwrap_or(url);
            let anchors = link_occurrences.get(url_without_fragment).cloned().unwrap_or_default();
            ExternalLink {
                url: url.to_string(),
                anchors,
            }
        })
        .collect();

    let sources: Vec<String> = post.sources.iter().map(|source_url| source_url.to_string()).collect();

    let bookmarked = RwSignal::new(is_bookmarked(post.slug));

    let slug_for_bookmark = post.slug.to_string();
    let toggle_bookmark = move |_| {
        let current = bookmarked.get();
        set_bookmarked(&slug_for_bookmark, !current);
        bookmarked.set(!current);
    };

    let scroll_to_top = move |_| {
        let _ = js_sys::eval("window.scrollTo({top: 0, behavior: 'smooth'})");
    };

    let (toc_html, content_html) = if post.toc() {
        let (toc, numbered) = generate_toc(&content_html);
        (Some(toc), numbered)
    } else {
        (None, content_html)
    };

    view! {
        <Title text=format!("\u{03bb} {} \u{2014} {}", META.title, post.title()) />
        <Meta name="description" content=post.description().to_string() />
        <Meta name="author" content=META.author />
        <div
            class="reading-progress"
            style:width=move || format!("{}%", scroll_progress.get().max(0.0))
            style:display=move || if scroll_progress.get() < 0.0 { "none" } else { "block" }
        />
        <Layout>
            <PostSearch />
            <article id="post-content">
                {post.is_draft().then(|| view! { <div class="draft-banner">"DRAFT"</div> })}
                <h1 class="post-title">
                    <button
                        class=move || {
                            if bookmarked.get() { "bookmark-btn active" } else { "bookmark-btn" }
                        }
                        on:click=toggle_bookmark
                        title=move || {
                            if bookmarked.get() { "remove bookmark" } else { "bookmark" }
                        }
                        inner_html=move || {
                            if bookmarked.get() {
                                "<svg width=\"14\" height=\"18\" viewBox=\"0 0 14 18\" fill=\"currentColor\" stroke=\"none\"><path d=\"M0 0h14v18l-7-4-7 4z\"/></svg>"
                            } else {
                                "<svg width=\"14\" height=\"18\" viewBox=\"0 0 14 18\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\"><path d=\"M1 1h12v15l-6-3.5L1 16z\"/></svg>"
                            }
                        }
                    />
                    <span class="post-title-text" inner_html=post.title() />
                    <button
                        class="print-btn"
                        on:click=|_| {
                            let _ = js_sys::eval("printPost()");
                        }
                        title="Print / Save as PDF"
                    >
                        "pdf"
                    </button>
                </h1>
                {(!post.description().is_empty())
                    .then(|| view! { <p class="post-subtitle" inner_html=post.description() /> })}
                <div class="post-meta">
                    {(post.last_edited() != post.date() && !post.last_edited().is_empty())
                        .then(|| {
                            view! {
                                <span class="date">
                                    {post.date_formatted()}{", last edited "}
                                    {post.last_edited_formatted()}
                                </span>
                            }
                        })}
                    <span class="post-tags">
                        {post
                            .tags()
                            .into_iter()
                            .map(|tag| {
                                let href = format!("/blog?tag={tag}");
                                view! {
                                    <A href=href attr:class="tag">
                                        {format!("#{tag}")}
                                    </A>
                                }
                            })
                            .collect_view()}
                    </span>
                </div> {render_series_navigation(post)}
                {toc_html
                    .map(|html| {
                        view! {
                            <details class="post-toc">
                                <summary>"Table of Contents"</summary>
                                <nav inner_html=html />
                            </details>
                        }
                    })} <div class="content" inner_html=content_html />
                {render_references(outgoing_links, incoming_links, external_links, sources)}
                {render_giscus()}
            </article>
            <button
                class="scroll-to-top"
                on:click=scroll_to_top
                style:display=move || if show_scroll_top.get() { "flex" } else { "none" }
                title="Scroll to top"
            >
                "\u{2191}"
            </button>
        </Layout>
    }
}

// ============================================================
// Table of Contents
// ============================================================

/// Extract headings from rendered HTML, returning (level, id, text) tuples.
fn extract_headings(content_html: &str) -> Vec<(u8, String, String)> {
    let mut headings = Vec::new();
    let mut remaining = content_html;

    while let Some(pos) = remaining.find("<h") {
        let after = &remaining[pos + 2..];
        let Some(level_char) = after.chars().next() else {
            remaining = after;
            continue;
        };
        let level = match level_char {
            '1' => 1,
            '2' => 2,
            '3' => 3,
            _ => {
                remaining = after;
                continue;
            }
        };

        let header_tag_end = after.find('>').unwrap_or(0);
        let header_tag = &after[..header_tag_end];
        let id = if let Some(id_start) = header_tag.find("id=\"") {
            let id_content = &header_tag[id_start + 4..];
            id_content.split('"').next().unwrap_or("").to_string()
        } else {
            String::new()
        };

        let content_start = header_tag_end + 1;
        let close_tag = format!("</h{level_char}>");
        if let Some(close_pos) = after[content_start..].find(&close_tag) {
            let raw_text = &after[content_start..content_start + close_pos];
            let text = strip_html_tags(raw_text);
            headings.push((level, id, text));
            remaining = &after[content_start + close_pos + close_tag.len()..];
        } else {
            remaining = after;
        }
    }
    return headings;
}

/// Build a section number string for the given counters and depth.
/// Uses relative indexing from the minimum heading level so numbering always starts at 1.
fn section_number(counters: &[u32], depth: usize) -> String {
    counters[..=depth]
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

/// Generate the TOC HTML and inject section numbers into content headings.
fn generate_toc(content_html: &str) -> (String, String) {
    let headings = extract_headings(content_html);
    if headings.is_empty() {
        return (String::from("<ul></ul>"), content_html.to_string());
    }

    // Determine the minimum heading level to use as the "major" level
    let min_level = headings.iter().map(|(l, _, _)| *l).min().unwrap_or(1);

    let mut toc = String::from("<ul>");
    let mut counters = [0u32; 3];

    // Also build a map of id → section number to inject into content
    let mut section_numbers: Vec<(String, String)> = Vec::new();

    for (level, id, text) in &headings {
        let relative_depth = (*level - min_level) as usize;
        let depth = relative_depth.min(2);

        counters[depth] += 1;
        for counter in &mut counters[depth + 1..] {
            *counter = 0;
        }

        let sec_num = section_number(&counters, depth);

        let class = match relative_depth {
            1 => " class=\"toc-h2\"",
            2.. => " class=\"toc-h3\"",
            _ => "",
        };

        if !id.is_empty() {
            toc.push_str(&format!(
                "<li{class}><a href=\"#{id}\"><span class=\"toc-num\">{sec_num}</span> {text}</a></li>"
            ));
            section_numbers.push((id.clone(), sec_num));
        } else {
            toc.push_str(&format!(
                "<li{class}><span class=\"toc-num\">{sec_num}</span> {text}</li>"
            ));
        }
    }
    toc.push_str("</ul>");

    // Inject section numbers into the actual content headings
    let mut numbered_content = content_html.to_string();
    for (id, sec_num) in section_numbers.iter().rev() {
        let search = format!("id=\"{id}\">");
        if let Some(pos) = numbered_content.find(&search) {
            let insert_pos = pos + search.len();
            numbered_content.insert_str(insert_pos, &format!("<span class=\"heading-num\">{sec_num}</span> "));
        }
    }

    return (toc, numbered_content);
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for character in html.chars() {
        if character == '<' {
            in_tag = true;
        } else if character == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(character);
        }
    }
    return result;
}

// ============================================================
// Series navigation
// ============================================================

fn render_series_navigation(post: &'static Post) -> Option<impl IntoView> {
    let series_name = post.series()?;
    let series_posts = post.series_posts();
    let current_index = series_posts
        .iter()
        .position(|series_post| series_post.slug == post.slug);
    let total = series_posts.len();

    let previous_post = current_index.and_then(|index| {
        if index > 0 {
            Some(&series_posts[index - 1])
        } else {
            None
        }
    });
    let next_post = current_index.and_then(|index| series_posts.get(index + 1));

    let part_number = current_index.map(|index| index + 1).unwrap_or(0);

    let table_of_contents_view = series_posts
        .iter()
        .enumerate()
        .map(|(i, series_post)| {
            let num = format!("{}.", i + 1);
            let href = series_post.href();
            let title = series_post.title().to_string();
            let is_current = series_post.slug == post.slug;
            if is_current {
                view! {
                    <li>
                        <span class="toc-num">{num}</span>
                        <strong>{title}</strong>
                    </li>
                }
                .into_any()
            } else {
                view! {
                    <li>
                        <span class="toc-num">{num}</span>
                        <A href=href>{title}</A>
                    </li>
                }
                .into_any()
            }
        })
        .collect_view();

    return Some(view! {
        <nav class="series-nav">
            <details>
                <summary>
                    {format!("Part {} of {} \u{2014} {}", part_number, total, series_name)}
                </summary>
                <ul class="series-toc">{table_of_contents_view}</ul>
            </details>
            <div class="series-prev-next">
                {previous_post
                    .map(|series_post| {
                        let href = series_post.href();
                        let label = format!("\u{25c2} {}", series_post.title());
                        view! {
                            <A href=href attr:class="series-prev">
                                {label}
                            </A>
                        }
                    })}
                {next_post
                    .map(|series_post| {
                        let href = series_post.href();
                        let label = format!("{} \u{25b8}", series_post.title());
                        view! {
                            <A href=href attr:class="series-next">
                                {label}
                            </A>
                        }
                    })}
            </div>
        </nav>
    });
}

// ============================================================
// References (outgoing, external, incoming, sources)
// ============================================================

fn render_backlinks(anchor_ids: Vec<String>) -> impl IntoView {
    if anchor_ids.is_empty() {
        return view! { <span /> }.into_any();
    }
    view! {
        <span class="ref-backlinks">
            {anchor_ids
                .into_iter()
                .enumerate()
                .map(|(index, anchor_id)| {
                    let href = format!("#{anchor_id}");
                    let label = format!("\u{2191}{}", index + 1);
                    view! {
                        <a href=href class="ref-backlink">
                            {label}
                        </a>
                    }
                })
                .collect_view()}
        </span>
    }
    .into_any()
}

fn render_post_link_list(links: Vec<PostLink>) -> impl IntoView {
    links
        .into_iter()
        .map(|link| {
            let title_display = link.title.clone();
            view! {
                <li>
                    <a
                        href=link.url
                        data-post-title=link.title
                        data-post-desc=link.description
                        data-post-tags=link.tags
                        data-post-series=link.series
                    >
                        {title_display}
                    </a>
                    {render_backlinks(link.anchors)}
                </li>
            }
        })
        .collect_view()
}

fn render_references(
    outgoing: Vec<PostLink>,
    incoming: Vec<PostLink>,
    external: Vec<ExternalLink>,
    sources: Vec<String>,
) -> impl IntoView {
    let has_any = !outgoing.is_empty() || !incoming.is_empty() || !external.is_empty() || !sources.is_empty();
    if !has_any {
        return view! { <div /> }.into_any();
    }

    return view! {
        <div class="post-references">
            {(!outgoing.is_empty())
                .then(|| {
                    view! {
                        <div class="ref-section">
                            <h2>"Internal references"</h2>
                            <ul>{render_post_link_list(outgoing)}</ul>
                        </div>
                    }
                })}
            {(!external.is_empty())
                .then(|| {
                    view! {
                        <div class="ref-section">
                            <h2>"External references"</h2>
                            <ul>
                                {external
                                    .into_iter()
                                    .map(|link| {
                                        let display = link.url.clone();
                                        view! {
                                            <li>
                                                <a href=link.url target="_blank">
                                                    {display}
                                                </a>
                                                {render_backlinks(link.anchors)}
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        </div>
                    }
                })}
            {(!sources.is_empty())
                .then(|| {
                    view! {
                        <div class="ref-section">
                            <h2>"Sources"</h2>
                            <ul>
                                {sources
                                    .into_iter()
                                    .map(|url| {
                                        let display = url.clone();
                                        view! {
                                            <li>
                                                <a href=url target="_blank">
                                                    {display}
                                                </a>
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        </div>
                    }
                })}
            {(!incoming.is_empty())
                .then(|| {
                    view! {
                        <div class="ref-section">
                            <h2>"Referenced internally by"</h2>
                            <ul>{render_post_link_list(incoming)}</ul>
                        </div>
                    }
                })}
        </div>
    }
    .into_any();
}

// ============================================================
// Giscus comments
// ============================================================

fn render_giscus() -> impl IntoView {
    let container_ref = NodeRef::<leptos::html::Div>::new();

    Effect::new(move |_: Option<()>| {
        let Some(container) = container_ref.get() else { return };
        let Some(window) = web_sys::window() else { return };
        let Some(document) = window.document() else { return };

        let theme = document
            .document_element()
            .and_then(|element| element.get_attribute("data-theme"))
            .unwrap_or_else(|| "dark".to_string());
        let giscus_theme = crate::components::giscus_theme_for(&theme);

        let script = document.create_element("script").unwrap();
        script.set_attribute("src", "https://giscus.app/client.js").unwrap();
        script.set_attribute("data-repo", "lsck0/lsck0.github.io").unwrap();
        script.set_attribute("data-repo-id", "R_kgDORX3_qQ").unwrap();
        script.set_attribute("data-category", "Comments").unwrap();
        script
            .set_attribute("data-category-id", "DIC_kwDORX3_qc4C4jik")
            .unwrap();
        script.set_attribute("data-mapping", "pathname").unwrap();
        script.set_attribute("data-strict", "1").unwrap();
        script.set_attribute("data-reactions-enabled", "1").unwrap();
        script.set_attribute("data-emit-metadata", "0").unwrap();
        script.set_attribute("data-input-position", "top").unwrap();
        script.set_attribute("data-theme", giscus_theme).unwrap();
        script.set_attribute("data-lang", "en").unwrap();
        script.set_attribute("data-loading", "lazy").unwrap();
        script.set_attribute("crossorigin", "anonymous").unwrap();
        script.set_attribute("async", "").unwrap();

        let element: &web_sys::Element = container.as_ref();
        element.set_inner_html("");
        element.append_child(&script).unwrap();
    });

    view! { <div class="giscus-container" node_ref=container_ref /> }
}
