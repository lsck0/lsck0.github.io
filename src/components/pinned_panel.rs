#![allow(clippy::needless_return)]

use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};

use super::render::markdown_to_html as render_markdown;

// ============================================================
// Pinned block data
// ============================================================

#[derive(Clone, ::serde::Deserialize, ::serde::Serialize)]
struct PinnedBlock {
    label: String,
    kind: String,
    title: String,
    number: String,
    preview: String,
    href: String,
}

fn read_pinned_blocks() -> Vec<PinnedBlock> {
    let Some(window) = web_sys::window() else { return vec![] };
    let Ok(Some(storage)) = window.local_storage() else { return vec![] };
    let Ok(Some(json)) = storage.get_item("pinned-blocks") else { return vec![] };
    return ::serde_json::from_str(&json).unwrap_or_default();
}

// ============================================================
// Component
// ============================================================

#[component]
pub fn PinnedPanel() -> impl IntoView {
    let (blocks, set_blocks) = signal(read_pinned_blocks());

    // Listen for changes from the JS pin/unpin buttons
    Effect::new(move |_: Option<()>| {
        let Some(window) = web_sys::window() else { return };
        let on_change = Closure::<dyn FnMut()>::new(move || {
            set_blocks.set(read_pinned_blocks());
        });
        let handler: js_sys::Function = on_change.into_js_value().unchecked_into();
        let _ = window.add_event_listener_with_callback("pinned-blocks-changed", &handler);
        on_cleanup(move || {
            let _ = window.remove_event_listener_with_callback("pinned-blocks-changed", &handler);
        });
    });

    // Re-render math after blocks update
    Effect::new(move |_: Option<()>| {
        let _ = blocks.get();
        request_animation_frame(move || {
            let _ = js_sys::eval(
                r#"(function() {
                    var panel = document.querySelector('.pinned-panel');
                    if (panel && window.renderMathInElement) {
                        renderMathInElement(panel, {
                            delimiters: [
                                { left: "\\(", right: "\\)", display: false },
                                { left: "\\[", right: "\\]", display: true },
                            ],
                            throwOnError: false,
                        });
                    }
                })()"#,
            );
        });
    });

    return view! {
        {move || {
            let pinned = blocks.get();
            if pinned.is_empty() {
                return None;
            }
            Some(view! {
                <div class="pinned-panel">
                    <div class="pinned-panel-header">
                        <span class="pinned-panel-title">"pinned"</span>
                        <button
                            class="pinned-panel-clear"
                            title="Unpin all"
                            on:click=move |_| {
                                let _ = js_sys::eval(
                                    "localStorage.setItem('pinned-blocks', '[]'); \
                                     window.dispatchEvent(new CustomEvent('pinned-blocks-changed'));",
                                );
                            }
                        >
                            "clear all"
                        </button>
                    </div>
                    <div class="pinned-panel-blocks">
                        {pinned
                            .into_iter()
                            .map(|block| {
                                let label = block.label.clone();
                                let kind_display = format!(
                                    "{}{}",
                                    capitalize(&block.kind),
                                    if block.number.is_empty() {
                                        String::new()
                                    } else {
                                        format!(" {}", block.number)
                                    },
                                );
                                let title_display = if block.title.is_empty() {
                                    None
                                } else {
                                    Some(block.title.clone())
                                };
                                let content_html = render_markdown(&block.preview).html;
                                let href = block.href.clone();
                                view! {
                                    <div class="pinned-block">
                                        <div class="pinned-block-header">
                                            <span class="pinned-block-kind">{kind_display}</span>
                                            {title_display
                                                .map(|t| {
                                                    view! {
                                                        <span class="pinned-block-title">
                                                            {format!(" ({t})")}
                                                        </span>
                                                    }
                                                })}
                                            <button
                                                class="pinned-block-unpin"
                                                title="Unpin"
                                                on:click={
                                                    let label = label.clone();
                                                    move |_| {
                                                        let _ = js_sys::eval(
                                                            &format!(
                                                                "removePinnedBlock('{}'); ",
                                                                label.replace('\'', "\\'"),
                                                            ),
                                                        );
                                                    }
                                                }
                                            >
                                                "\u{2715}"
                                            </button>
                                        </div>
                                        <div class="pinned-block-content" inner_html=content_html></div>
                                        <a class="pinned-block-link" href=href>
                                            "go to source \u{2192}"
                                        </a>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            })
        }}
    };
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
