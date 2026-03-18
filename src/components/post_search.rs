#![allow(clippy::needless_return)]

use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};

// ============================================================
// Component
// ============================================================

#[component]
pub fn PostSearch() -> impl IntoView {
    let (is_visible, set_visible) = signal(false);
    let (search_query, set_search_query) = signal(String::new());
    let (total_matches, set_total_matches) = signal(0usize);
    let (active_match_index, set_active_match_index) = signal(0usize);
    let search_input_reference = NodeRef::<leptos::html::Input>::new();

    register_keyboard_shortcuts(
        is_visible,
        set_visible,
        set_search_query,
        set_total_matches,
        search_input_reference,
    );

    track_query_and_highlight(search_query, set_total_matches, set_active_match_index);

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

    let close_search = move |_| {
        set_visible.set(false);
        clear_highlights();
        set_search_query.set(String::new());
        set_total_matches.set(0);
    };

    return view! {
        <div class="post-search" style:display=move || if is_visible.get() { "flex" } else { "none" }>
            <input
                type="text"
                placeholder="search in post..."
                prop:value=search_query
                on:input=move |event| set_search_query.set(event_target_value(&event))
                node_ref=search_input_reference
            />
            <span class="search-count">
                {move || format_match_counter(active_match_index.get(), total_matches.get(), search_query.get().len())}
            </span>
            <button on:click=navigate_to_previous_match title="Previous match">
                {"\u{25b2}"}
            </button>
            <button on:click=navigate_to_next_match title="Next match">
                {"\u{25bc}"}
            </button>
            <button on:click=close_search title="Close (Esc)">
                {"\u{2715}"}
            </button>
        </div>
    };
}

// ============================================================
// Effects
// ============================================================

fn register_keyboard_shortcuts(
    is_visible: ReadSignal<bool>,
    set_visible: WriteSignal<bool>,
    set_search_query: WriteSignal<String>,
    set_total_matches: WriteSignal<usize>,
    search_input_reference: NodeRef<leptos::html::Input>,
) {
    Effect::new(move |_: Option<()>| {
        let Some(window) = web_sys::window() else { return };

        let on_keydown = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            let is_toggle_shortcut = (event.ctrl_key() || event.meta_key()) && event.shift_key() && event.key() == "f";

            if is_toggle_shortcut {
                event.prevent_default();
                set_visible.update(|visible| *visible = !*visible);

                if is_visible.get_untracked() {
                    leptos::task::spawn_local(async move {
                        if let Some(input) = search_input_reference.get() {
                            let _ = input.focus();
                        }
                    });
                } else {
                    reset_search(set_search_query, set_total_matches);
                }
            }

            if event.key() == "Escape" && is_visible.get_untracked() {
                set_visible.set(false);
                reset_search(set_search_query, set_total_matches);
            }
        });

        let keydown_handler: js_sys::Function = on_keydown.into_js_value().unchecked_into();
        window
            .add_event_listener_with_callback("keydown", &keydown_handler)
            .unwrap();

        on_cleanup(move || {
            if let Some(window) = web_sys::window() {
                let _ = window.remove_event_listener_with_callback("keydown", &keydown_handler);
            }
        });
    });
}

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

fn reset_search(set_search_query: WriteSignal<String>, set_total_matches: WriteSignal<usize>) {
    clear_highlights();
    set_search_query.set(String::new());
    set_total_matches.set(0);
}

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
