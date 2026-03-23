#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::{JsCast, closure::Closure};

use crate::models::{post::POSTS, search::fuzzy_score};

// ============================================================
// Component
// ============================================================

#[component]
pub fn PostSearch() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());
    let (total_matches, set_total_matches) = signal(0usize);
    let (active_match_index, set_active_match_index) = signal(0usize);
    let (is_visible, set_is_visible) = signal(false);
    let search_input_reference = NodeRef::<leptos::html::Input>::new();

    track_query_and_highlight(search_query, set_total_matches, set_active_match_index);

    // Keyboard shortcuts
    {
        Effect::new(move |_: Option<()>| {
            let Some(window) = web_sys::window() else { return };
            let on_keydown = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
                let key = event.key();
                let ctrl = event.ctrl_key() || event.meta_key();

                // Ctrl+F: open/focus in-page search
                if ctrl && !event.shift_key() && (key == "f" || key == "F") {
                    event.prevent_default();
                    set_is_visible.set(true);
                    // Defer focus to next frame so the input is rendered
                    let input_ref = search_input_reference;
                    request_animation_frame(move || {
                        if let Some(input) = input_ref.get() {
                            let _ = input.focus();
                            input.select();
                        }
                    });
                }

                // Escape: close search
                if key == "Escape" && is_visible.get() {
                    event.prevent_default();
                    set_is_visible.set(false);
                    set_search_query.set(String::new());
                    clear_highlights();
                }

                // Ctrl+N or Enter: next match (vim-like)
                if is_visible.get() && ((ctrl && key == "n") || (key == "Enter" && !event.shift_key())) {
                    event.prevent_default();
                    let count = total_matches.get();
                    if count > 0 {
                        let next = active_match_index.get() % count + 1;
                        set_active_match_index.set(next);
                        scroll_to_match(next - 1);
                    }
                }

                // Ctrl+Shift+N or Shift+Enter: previous match (vim-like)
                if is_visible.get() && ((ctrl && key == "N") || (key == "Enter" && event.shift_key())) {
                    event.prevent_default();
                    let count = total_matches.get();
                    if count > 0 {
                        let current = active_match_index.get();
                        let prev = if current <= 1 { count } else { current - 1 };
                        set_active_match_index.set(prev);
                        scroll_to_match(prev - 1);
                    }
                }
            });
            let keydown_handler: js_sys::Function = on_keydown.into_js_value().unchecked_into();
            let _ = window.add_event_listener_with_callback("keydown", &keydown_handler);
            on_cleanup(move || {
                let _ = window.remove_event_listener_with_callback("keydown", &keydown_handler);
            });
        });
    }

    let navigate_to_next_match = move |_| {
        let count = total_matches.get();
        if count == 0 {
            return;
        }
        let next_index = active_match_index.get() % count + 1;
        set_active_match_index.set(next_index);
        scroll_to_match(next_index - 1);
    };

    let navigate_to_previous_match = move |_| {
        let count = total_matches.get();
        if count == 0 {
            return;
        }
        let current = active_match_index.get();
        let previous_index = if current <= 1 { count } else { current - 1 };
        set_active_match_index.set(previous_index);
        scroll_to_match(previous_index - 1);
    };

    return view! {
        {move || {
            is_visible
                .get()
                .then(|| {
                    view! {
                        <div class="post-search">
                            <input
                                type="text"
                                placeholder="search in post..."
                                prop:value=search_query
                                on:input=move |event| {
                                    set_search_query.set(event_target_value(&event))
                                }
                                node_ref=search_input_reference
                            />
                            <span class="search-count">
                                {move || format_match_counter(
                                    active_match_index.get(),
                                    total_matches.get(),
                                    search_query.get().len(),
                                )}
                            </span>
                            <button on:click=navigate_to_previous_match title="Previous match">
                                {"\u{25b2}"}
                            </button>
                            <button on:click=navigate_to_next_match title="Next match">
                                {"\u{25bc}"}
                            </button>
                        </div>
                    }
                })
        }}
    };
}

// ============================================================
// Effects
// ============================================================

fn track_query_and_highlight(
    search_query: ReadSignal<String>,
    set_total_matches: WriteSignal<usize>,
    set_active_match_index: WriteSignal<usize>,
) {
    Effect::new(move |_: Option<()>| {
        let query_text = search_query.get();
        clear_highlights();

        if query_text.len() < 2 {
            set_total_matches.set(0);
            set_active_match_index.set(0);
            return;
        }

        let count = highlight_matches(&query_text);
        set_total_matches.set(count);
        set_active_match_index.set(if count > 0 { 1 } else { 0 });

        if count > 0 {
            scroll_to_match(0);
        }
    });
}

// ============================================================
// Helpers
// ============================================================

fn format_match_counter(active_index: usize, total: usize, query_length: usize) -> String {
    if total > 0 {
        return format!("{active_index}/{total}");
    }
    if query_length >= 2 {
        return "0".to_string();
    }
    return String::new();
}

// ============================================================
// DOM manipulation via JavaScript
// ============================================================

fn clear_highlights() {
    let _ = js_sys::eval(
        r#"document.querySelectorAll('mark.post-search-highlight').forEach(function(mark) {
            var parent = mark.parentNode;
            parent.replaceChild(document.createTextNode(mark.textContent), mark);
            parent.normalize();
        })"#,
    );
}

fn highlight_matches(search_text: &str) -> usize {
    let escaped_query = search_text
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n");

    let result = js_sys::eval(&format!(
        r#"(function() {{
            var postElement = document.getElementById('post-content');
            if (!postElement) return 0;
            var contentElement = postElement.querySelector('.content');
            if (!contentElement) return 0;
            var searchQuery = '{escaped_query}'.toLowerCase();
            var matchCount = 0;
            var walker = document.createTreeWalker(contentElement, NodeFilter.SHOW_TEXT, null);
            var foundMatches = [];
            while (walker.nextNode()) {{
                var textNode = walker.currentNode;
                var parentElement = textNode.parentElement;
                var isInsideCodeOrScript = parentElement
                    && (parentElement.tagName === 'SCRIPT'
                        || parentElement.tagName === 'STYLE'
                        || parentElement.tagName === 'CODE'
                        || parentElement.tagName === 'PRE');
                if (isInsideCodeOrScript) continue;
                var nodeText = textNode.textContent.toLowerCase();
                var position = nodeText.indexOf(searchQuery);
                while (position !== -1) {{
                    foundMatches.push({{ node: textNode, index: position }});
                    position = nodeText.indexOf(searchQuery, position + 1);
                }}
            }}
            for (var i = foundMatches.length - 1; i >= 0; i--) {{
                var match = foundMatches[i];
                var range = document.createRange();
                range.setStart(match.node, match.index);
                range.setEnd(match.node, match.index + searchQuery.length);
                var highlightMark = document.createElement('mark');
                highlightMark.className = 'post-search-highlight';
                range.surroundContents(highlightMark);
                matchCount++;
            }}
            return matchCount;
        }})()"#,
    ));

    return result
        .ok()
        .and_then(|value| value.as_f64())
        .map(|number| number as usize)
        .unwrap_or(0);
}

fn scroll_to_match(index: usize) {
    let _ = js_sys::eval(&format!(
        r#"(function() {{
            var highlights = document.querySelectorAll('mark.post-search-highlight');
            highlights.forEach(function(mark) {{ mark.classList.remove('current'); }});
            if (highlights[{index}]) {{
                highlights[{index}].classList.add('current');
                highlights[{index}].scrollIntoView({{ behavior: 'smooth', block: 'center' }});
            }}
        }})()"#,
    ));
}

// ============================================================
// Global search (Ctrl+Shift+F) — searches across ALL posts
// ============================================================

#[component]
pub fn GlobalSearch() -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (is_visible, set_is_visible) = signal(false);
    let (selected_index, set_selected_index) = signal(0usize);
    let search_input_ref = NodeRef::<leptos::html::Input>::new();
    let navigate = use_navigate();
    let nav_for_effect = navigate.clone();

    let results = move || {
        let q = query.get();
        if q.len() < 2 {
            return Vec::new();
        }
        let mut scored: Vec<(usize, u32)> = POSTS
            .iter()
            .enumerate()
            .filter_map(|(i, post)| {
                let title_score = fuzzy_score(&q, post.title()).map(|s| s.saturating_mul(3));
                let desc_score = fuzzy_score(&q, post.description());
                let block_text = post.labeled_block_text();
                let block_score = fuzzy_score(&q, &block_text).map(|s| s.saturating_mul(2));
                let best = [title_score, desc_score, block_score].into_iter().flatten().max();
                best.map(|score| (i, score))
            })
            .collect();
        scored.sort_by_key(|e| std::cmp::Reverse(e.1));
        scored.truncate(10);
        scored
    };

    // Keyboard shortcuts
    Effect::new(move |_: Option<()>| {
        let Some(window) = web_sys::window() else { return };
        let navigate = nav_for_effect.clone();
        let on_keydown = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            let key = event.key();
            let ctrl = event.ctrl_key() || event.meta_key();

            // Ctrl+Shift+F: open global search
            if ctrl && event.shift_key() && (key == "f" || key == "F") {
                event.prevent_default();
                set_is_visible.set(true);
                let input_ref = search_input_ref;
                request_animation_frame(move || {
                    if let Some(input) = input_ref.get() {
                        let _ = input.focus();
                        input.select();
                    }
                });
            }

            if !is_visible.get() {
                return;
            }

            // Escape: close
            if key == "Escape" {
                event.prevent_default();
                set_is_visible.set(false);
                set_query.set(String::new());
            }

            // Ctrl+N or Down: next result
            if (ctrl && key == "n") || key == "ArrowDown" {
                event.prevent_default();
                set_selected_index.update(|i| *i = i.saturating_add(1).min(9));
            }

            // Ctrl+Shift+N or Up: previous result
            if (ctrl && key == "N") || key == "ArrowUp" {
                event.prevent_default();
                set_selected_index.update(|i| *i = i.saturating_sub(1));
            }

            // Enter: navigate to selected post
            if key == "Enter" {
                event.prevent_default();
                let r = results();
                let idx = selected_index.get();
                if let Some(&(post_idx, _)) = r.get(idx)
                    && let Some(post) = POSTS.get(post_idx)
                {
                    let href = post.href();
                    set_is_visible.set(false);
                    set_query.set(String::new());
                    navigate(&href, Default::default());
                }
            }
        });
        let handler: js_sys::Function = on_keydown.into_js_value().unchecked_into();
        let _ = window.add_event_listener_with_callback("keydown", &handler);
        on_cleanup(move || {
            let _ = window.remove_event_listener_with_callback("keydown", &handler);
        });
    });

    // Reset selected index when query changes
    Effect::new(move |_: Option<()>| {
        let _ = query.get();
        set_selected_index.set(0);
    });

    let nav_for_view = navigate.clone();
    return view! {
        {move || {
            is_visible
                .get()
                .then(|| {
                    let nav = nav_for_view.clone();
                    view! {
                        <div
                            class="global-search-overlay"
                            on:click=move |_| {
                                set_is_visible.set(false);
                                set_query.set(String::new());
                            }
                        >
                            <div
                                class="global-search"
                                on:click=move |event: web_sys::MouseEvent| event.stop_propagation()
                            >
                                <input
                                    type="text"
                                    placeholder="search all posts..."
                                    prop:value=query
                                    on:input=move |event| set_query.set(event_target_value(&event))
                                    node_ref=search_input_ref
                                />
                                <div class="global-search-results">
                                    {move || {
                                        let r = results();
                                        let sel = selected_index.get();
                                        if r.is_empty() && query.get().len() >= 2 {
                                            return view! {
                                                <div class="global-search-empty">"no matches"</div>
                                            }
                                                .into_any();
                                        }
                                        r.into_iter()
                                            .enumerate()
                                            .map(|(i, (post_idx, _score))| {
                                                let post = &POSTS[post_idx];
                                                let class = if i == sel {
                                                    "global-search-item selected"
                                                } else {
                                                    "global-search-item"
                                                };
                                                let href = post.href();
                                                let nav = nav.clone();
                                                view! {
                                                    <div
                                                        class=class
                                                        on:click=move |_| {
                                                            let h = href.clone();
                                                            set_is_visible.set(false);
                                                            set_query.set(String::new());
                                                            nav(&h, Default::default());
                                                        }
                                                    >
                                                        <span class="global-search-title">{post.title()}</span>
                                                        <span class="global-search-slug">{post.slug}</span>
                                                    </div>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }}
                                </div>
                            </div>
                        </div>
                    }
                })
        }}
    };
}
