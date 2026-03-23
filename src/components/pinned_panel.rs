#![allow(clippy::needless_return)]

use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};

use super::capitalize;

// ============================================================
// Pinned block data
// ============================================================

#[derive(Clone, ::serde::Deserialize, ::serde::Serialize)]
struct PinnedBlock {
    label: String,
    kind: String,
    title: String,
    number: String,
    /// Stored as rendered HTML (not markdown).
    preview: String,
    href: String,
}

fn read_pinned_blocks() -> Vec<PinnedBlock> {
    let Some(window) = web_sys::window() else { return vec![] };
    let Ok(Some(storage)) = window.local_storage() else {
        return vec![];
    };
    let Ok(Some(json)) = storage.get_item("pinned-blocks") else {
        return vec![];
    };
    return ::serde_json::from_str(&json).unwrap_or_default();
}

fn set_has_pins_attribute(has_pins: bool) {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let Some(html) = document.document_element() else {
        return;
    };
    if has_pins {
        let _ = html.set_attribute("data-has-pins", "");
    } else {
        let _ = html.remove_attribute("data-has-pins");
    }
}

// ============================================================
// Component
// ============================================================

#[component]
pub fn PinnedPanel() -> impl IntoView {
    let (blocks, set_blocks) = signal(read_pinned_blocks());
    let (is_mobile_open, set_is_mobile_open) = signal(false);
    let (is_study_mode, set_is_study_mode) = signal(false);
    let (study_index, set_study_index) = signal(0usize);
    let (study_revealed, set_study_revealed) = signal(false);

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

    // Set data-has-pins attribute on html, clamp study index when blocks shrink
    Effect::new(move |_: Option<()>| {
        let pinned = blocks.get();
        set_has_pins_attribute(!pinned.is_empty());
        let max_index = pinned.len().saturating_sub(1);
        if study_index.get_untracked() > max_index {
            set_study_index.set(max_index);
        }
    });

    // Remove data-has-pins when component unmounts (e.g. navigating to home page)
    on_cleanup(move || {
        set_has_pins_attribute(false);
    });

    // Re-render math and tooltips in the panel whenever content changes
    // (blocks change, study card navigation, or reveal)
    Effect::new(move |_: Option<()>| {
        // Track all signals that change visible panel content
        let _ = blocks.get();
        let _ = study_index.get();
        let _ = study_revealed.get();
        let _ = is_study_mode.get();
        request_animation_frame(move || {
            let _ = js_sys::eval(
                r#"(function() {
                    var panel = document.querySelector('.pinned-panel');
                    if (!panel) return;
                    if (window.renderMathInElement) {
                        renderMathInElement(panel, {
                            delimiters: [
                                { left: "\\(", right: "\\)", display: false },
                                { left: "\\[", right: "\\]", display: true },
                            ],
                            throwOnError: false,
                        });
                    }
                    if (window.setupTooltips) setupTooltips(panel);
                })()"#,
            );
        });
    });

    let close_mobile = move |_| set_is_mobile_open.set(false);

    return view! {
        // Mobile toggle button (only visible when pins exist on mobile)
        {move || {
            let has_pins = !blocks.get().is_empty();
            has_pins
                .then(|| {
                    view! {
                        <button
                            class="pins-mobile-toggle"
                            on:click=move |_| set_is_mobile_open.update(|open| *open = !*open)
                            title="Show pinned blocks"
                        >
                            {move || {
                                let count = blocks.get().len();
                                format!("\u{1f4cc}{count}")
                            }}
                        </button>
                    }
                })
        }}
        {move || {
            let pinned = blocks.get();
            if pinned.is_empty() {
                return None;
            }
            let studying = is_study_mode.get();
            Some(
                view! {
                    <div class="pinned-panel" class:mobile-open=move || is_mobile_open.get()>
                        <div class="pinned-panel-header">
                            <span class="pinned-panel-title">"pinned"</span>
                            <div class="pinned-panel-actions">
                                <button
                                    class="pinned-panel-study"
                                    class:active=move || is_study_mode.get()
                                    title="Study mode (flashcards)"
                                    on:click=move |_| {
                                        let entering = !is_study_mode.get();
                                        set_is_study_mode.set(entering);
                                        if entering {
                                            set_study_index.set(0);
                                            set_study_revealed.set(false);
                                        }
                                    }
                                >
                                    "study"
                                </button>
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
                                <button class="pinned-panel-close-mobile" on:click=close_mobile>
                                    "\u{2715}"
                                </button>
                            </div>
                        </div>
                        {if studying {
                            render_study_mode(
                                    pinned,
                                    study_index,
                                    set_study_index,
                                    study_revealed,
                                    set_study_revealed,
                                )
                                .into_any()
                        } else {
                            render_block_list(pinned).into_any()
                        }}
                    </div>
                },
            )
        }}
    };
}

// ============================================================
// Block list (normal mode)
// ============================================================

fn render_block_list(pinned: Vec<PinnedBlock>) -> impl IntoView {
    view! {
        <div class="pinned-panel-blocks">
            {pinned
                .into_iter()
                .map(|block| {
                    let label = block.label.clone();
                    let is_proof = block.kind == "proof";
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
                    let content_html = block.preview.clone();
                    let href = block.href.clone();
                    view! {
                        <div class="pinned-block">
                            <div class="pinned-block-header">
                                <span class="pinned-block-kind">{kind_display}</span>
                                {title_display
                                    .map(|t| {
                                        let prefix = if is_proof { " of " } else { " (" };
                                        let suffix = if is_proof { "" } else { ")" };
                                        view! {
                                            <span class="pinned-block-title">
                                                {format!("{prefix}{t}{suffix}")}
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
    }
}

// ============================================================
// Study mode (Anki-style flashcards)
// ============================================================

fn render_study_mode(
    pinned: Vec<PinnedBlock>,
    study_index: ReadSignal<usize>,
    set_study_index: WriteSignal<usize>,
    study_revealed: ReadSignal<bool>,
    set_study_revealed: WriteSignal<bool>,
) -> impl IntoView {
    let total = pinned.len();
    if total == 0 {
        return view! { <div class="study-empty">"no cards to study"</div> }.into_any();
    }

    // Derive card content reactively from study_index
    let pinned_clone = pinned.clone();
    let card_title = Memo::new(move |_| {
        let idx = study_index.get().min(total.saturating_sub(1));
        let block = &pinned_clone[idx];
        let is_proof = block.kind == "proof";
        let kind_display = format!(
            "{}{}",
            capitalize(&block.kind),
            if block.number.is_empty() {
                String::new()
            } else {
                format!(" {}", block.number)
            },
        );
        if block.title.is_empty() {
            kind_display
        } else if is_proof {
            format!("{kind_display} of {title}", title = block.title)
        } else {
            format!("{kind_display} ({title})", title = block.title)
        }
    });

    let pinned_clone2 = pinned.clone();
    let card_html = Memo::new(move |_| {
        let idx = study_index.get().min(total.saturating_sub(1));
        pinned_clone2[idx].preview.clone()
    });

    let is_first = move || study_index.get() == 0;
    // Note: cannot use >= in RSX (> closes the tag), so use equivalent form
    #[allow(clippy::nonminimal_bool)]
    let is_last = move || !(study_index.get() + 1 < total);

    view! {
        <div class="study-card">
            <div class="study-progress">
                {move || format!("{}/{}", study_index.get() + 1, total)}
            </div>
            <div class="study-front">
                <span class="study-kind">{move || card_title.get()}</span>
            </div>
            <div
                class="study-back"
                class:revealed=move || study_revealed.get()
                inner_html=move || card_html.get()
            ></div>
            {move || {
                if !study_revealed.get() {
                    view! {
                        <button
                            class="study-reveal-btn"
                            on:click=move |_| set_study_revealed.set(true)
                        >
                            "reveal"
                        </button>
                    }
                        .into_any()
                } else {
                    view! {
                        <div class="study-nav">
                            <button
                                class="study-nav-btn"
                                disabled=is_first
                                on:click=move |_| {
                                    set_study_index.update(|i| *i = i.saturating_sub(1));
                                    set_study_revealed.set(false);
                                }
                            >
                                "\u{2190} prev"
                            </button>
                            <button
                                class="study-nav-btn"
                                disabled=is_last
                                on:click=move |_| {
                                    set_study_index.update(|i| *i += 1);
                                    set_study_revealed.set(false);
                                }
                            >
                                "next \u{2192}"
                            </button>
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
    .into_any()
}
