use std::collections::HashMap;

use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};

use super::storage::{self, PinnedBlock, StudyScore};

// ============================================================
// Helpers
// ============================================================

fn format_block_heading(block: &PinnedBlock) -> String {
    let is_proof = block.kind == "proof";
    let kind = format!(
        "{}{}",
        ir::capitalize(&block.kind),
        if block.number.is_empty() {
            String::new()
        } else {
            format!(" {}", block.number)
        },
    );
    if block.title.is_empty() {
        kind
    } else if is_proof {
        format!("{kind} of {title}", title = block.title)
    } else {
        format!("{kind} ({title})", title = block.title)
    }
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

/// Order blocks so those with more wrong answers appear first (spaced repetition).
fn study_order(blocks: &[PinnedBlock], scores: &HashMap<String, StudyScore>) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..blocks.len()).collect();
    indices.sort_by(|&a, &b| {
        let sa = scores.get(&blocks[a].label).cloned().unwrap_or_default();
        let sb = scores.get(&blocks[b].label).cloned().unwrap_or_default();
        // Higher wrong ratio = earlier in the queue
        let ratio_a = if sa.right + sa.wrong == 0 {
            0.5 // unseen cards go in the middle
        } else {
            sa.wrong as f64 / (sa.right + sa.wrong) as f64
        };
        let ratio_b = if sb.right + sb.wrong == 0 {
            0.5
        } else {
            sb.wrong as f64 / (sb.right + sb.wrong) as f64
        };
        ratio_b.partial_cmp(&ratio_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    return indices;
}

// ============================================================
// Re-render math/tooltips in a container
// ============================================================

fn rerender_math_in(selector: &str) {
    let script = format!(
        r#"(function() {{
            var el = document.querySelector('{selector}');
            if (!el) return;
            if (window.renderMathInElement) {{
                renderMathInElement(el, {{
                    delimiters: [
                        {{ left: "\\(", right: "\\)", display: false }},
                        {{ left: "\\[", right: "\\]", display: true }},
                    ],
                    throwOnError: false,
                }});
            }}
            if (window.setupTooltips) setupTooltips(el);
        }})()"#,
    );
    request_animation_frame(move || {
        let _ = js_sys::eval(&script);
    });
}

// ============================================================
// Component
// ============================================================

#[component]
pub fn PinnedPanel() -> impl IntoView {
    let (blocks, set_blocks) = signal(storage::read_pinned_blocks());
    let (is_mobile_open, set_is_mobile_open) = signal(false);
    let (is_desktop_open, set_is_desktop_open) = signal(true);
    let (is_study_open, set_is_study_open) = signal(false);

    // Listen for changes from the JS pin/unpin buttons
    Effect::new(move |_: Option<()>| {
        let Some(window) = web_sys::window() else { return };
        let on_change = Closure::<dyn FnMut()>::new(move || {
            set_blocks.set(storage::read_pinned_blocks());
        });
        let handler: js_sys::Function = on_change.into_js_value().unchecked_into();
        let _ = window.add_event_listener_with_callback("pinned-blocks-changed", &handler);
        on_cleanup(move || {
            let _ = window.remove_event_listener_with_callback("pinned-blocks-changed", &handler);
        });
    });

    // Set data-has-pins attribute on html element (only when desktop panel is open)
    Effect::new(move |_: Option<()>| {
        set_has_pins_attribute(!blocks.get().is_empty() && is_desktop_open.get());
    });

    // Deferred cleanup: clear data-has-pins when PinnedPanel unmounts (e.g. navigating
    // to HomePage which has no Layout). Checks if a pinned-panel still exists in DOM —
    // if a new PinnedPanel mounted (Layout→Layout nav), the element is present and we
    // skip clearing so the new component's Effect stays in control.
    on_cleanup(move || {
        let _ = web_sys::window().map(|w| {
            let cb = Closure::<dyn FnMut()>::once(move || {
                let panel_exists = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.query_selector(".pinned-panel").ok().flatten())
                    .is_some();
                if !panel_exists {
                    set_has_pins_attribute(false);
                }
            });
            let _ = w.set_timeout_with_callback(cb.as_ref().unchecked_ref());
            cb.forget();
        });
    });

    // Re-render math in the pinned panel when blocks change
    Effect::new(move |_: Option<()>| {
        let _ = blocks.get();
        rerender_math_in(".pinned-panel");
    });

    let close_mobile = move |_| set_is_mobile_open.set(false);

    return view! {
        // Mobile toggle button
        {move || {
            (!blocks.get().is_empty())
                .then(|| {
                    view! {
                        <button
                            class="pins-mobile-toggle"
                            on:click=move |_| set_is_mobile_open.update(|open| *open = !*open)
                            title="Show pinned blocks"
                        >
                            {move || format!("\u{1f4cc}{}", blocks.get().len())}
                        </button>
                    }
                })
        }}

        // Desktop toggle button (visible when panel is closed)
        {move || {
            let has_pins = !blocks.get().is_empty();
            let desktop_closed = !is_desktop_open.get();
            (has_pins && desktop_closed)
                .then(|| {
                    view! {
                        <button
                            class="pins-desktop-toggle"
                            on:click=move |_| set_is_desktop_open.set(true)
                            title="Show pinned blocks"
                        >
                            {move || format!("\u{1f4cc}{}", blocks.get().len())}
                        </button>
                    }
                })
        }}

        // Pinned panel
        {move || {
            let pinned = blocks.get();
            if pinned.is_empty() {
                return None;
            }
            Some(
                view! {
                    <div
                        class="pinned-panel"
                        class:mobile-open=move || is_mobile_open.get()
                        class:desktop-open=move || is_desktop_open.get()
                    >
                        <div class="pinned-panel-header">
                            <span class="pinned-panel-title">"pinned"</span>
                            <div class="pinned-panel-actions">
                                <button
                                    class="pinned-panel-study"
                                    title="Study mode (flashcards)"
                                    on:click=move |_| set_is_study_open.set(true)
                                >
                                    "study"
                                </button>
                                <button
                                    class="pinned-panel-clear"
                                    title="Unpin all"
                                    on:click=move |_| {
                                        storage::clear_pinned_blocks();
                                    }
                                >
                                    "clear all"
                                </button>
                                <button
                                    class="pinned-panel-close"
                                    on:click=move |_| set_is_desktop_open.set(false)
                                    title="Close pinned panel"
                                >
                                    "\u{2715}"
                                </button>
                                <button class="pinned-panel-close-mobile" on:click=close_mobile>
                                    "\u{2715}"
                                </button>
                            </div>
                        </div>
                        {render_block_list(pinned)}
                    </div>
                },
            )
        }}

        // Study mode modal (overlay popup)
        <StudyModal is_open=is_study_open set_is_open=set_is_study_open blocks=blocks />
    };
}

// ============================================================
// Block list (pinned panel content)
// ============================================================

fn render_block_list(pinned: Vec<PinnedBlock>) -> impl IntoView {
    view! {
        <div class="pinned-panel-blocks">
            {pinned
                .into_iter()
                .map(|block| {
                    let label = block.label.clone();
                    let heading = format_block_heading(&block);
                    let content_html = block.preview.clone();
                    let href = block.href.clone();
                    view! {
                        <div class="pinned-block">
                            <div class="pinned-block-header">
                                <span class="pinned-block-kind">{heading}</span>
                                <button
                                    class="pinned-block-unpin"
                                    title="Unpin"
                                    on:click={
                                        let label = label.clone();
                                        move |_| {
                                            storage::remove_pinned_block(&label);
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
// Study modal (popup with spaced repetition)
// ============================================================

#[component]
fn StudyModal(
    is_open: ReadSignal<bool>,
    set_is_open: WriteSignal<bool>,
    blocks: ReadSignal<Vec<PinnedBlock>>,
) -> impl IntoView {
    let (revealed, set_revealed) = signal(false);
    let (scores, set_scores) = signal(storage::read_study_scores());
    // Session deck — a queue of block indices; cards are re-inserted based on answers
    let (deck, set_deck) = signal(std::collections::VecDeque::<usize>::new());
    // Track consecutive "knew it" count per card within this session
    let (knew_streak, set_knew_streak) = signal(HashMap::<usize, u32>::new());
    let (cards_completed, set_cards_completed) = signal(0usize);

    // Build initial deck when modal opens
    Effect::new(move |_: Option<()>| {
        if is_open.get() {
            set_revealed.set(false);
            let sc = storage::read_study_scores();
            let pinned = blocks.get_untracked();
            let order = study_order(&pinned, &sc);
            set_deck.set(order.into());
            set_knew_streak.set(HashMap::new());
            set_scores.set(sc);
            set_cards_completed.set(0);
        }
    });

    // Re-render math when the front card changes or is revealed
    Effect::new(move |_: Option<()>| {
        let _ = deck.get();
        let _ = revealed.get();
        if is_open.get() {
            rerender_math_in(".study-modal");
        }
    });

    // Escape key to close
    Effect::new(move |_: Option<()>| {
        if !is_open.get() {
            return;
        }
        let Some(window) = web_sys::window() else { return };
        let on_keydown = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            if event.key() == "Escape" {
                set_is_open.set(false);
            }
        });
        let handler: js_sys::Function = on_keydown.into_js_value().unchecked_into();
        let _ = window.add_event_listener_with_callback("keydown", &handler);
        on_cleanup(move || {
            let _ = window.remove_event_listener_with_callback("keydown", &handler);
        });
    });

    // answer: 0 = didn't know, 1 = unsure, 2 = knew it
    let handle_answer = move |answer: u8| {
        let pinned = blocks.get_untracked();
        let mut current_deck = deck.get_untracked();

        let Some(block_idx) = current_deck.pop_front() else {
            return;
        };

        // Record score to persistent storage
        if let Some(block) = pinned.get(block_idx) {
            let mut sc = scores.get_untracked();
            let entry = sc.entry(block.label.clone()).or_default();
            match answer {
                0 => entry.wrong += 1,
                1 => entry.unsure += 1,
                _ => entry.right += 1,
            }
            storage::save_study_scores(&sc);
            set_scores.set(sc);
        }

        // Deck re-insertion logic
        match answer {
            0 => {
                // "didn't know" — insert 2 copies (near-front + mid-front) → ~3x frequency
                set_knew_streak.update(|m| {
                    m.remove(&block_idx);
                });
                // First copy: near the front (position 2 or end if deck is tiny)
                let pos1 = current_deck.len().min(2);
                current_deck.insert(pos1, block_idx);
                // Second copy: ~1/3 into the deck (recalc len since we just inserted)
                let len = current_deck.len();
                let pos2 = len.min(len / 3 + 3);
                current_deck.insert(pos2, block_idx);
            }
            1 => {
                // "unsure" — insert 1 copy in middle of deck → normal frequency
                set_knew_streak.update(|m| {
                    m.remove(&block_idx);
                });
                let pos = current_deck.len() / 2;
                current_deck.insert(pos, block_idx);
            }
            _ => {
                // "knew it" — check consecutive streak
                let streak = knew_streak.get_untracked().get(&block_idx).copied().unwrap_or(0) + 1;
                if streak >= 2 {
                    // Second consecutive "knew it" — card leaves the deck
                    set_knew_streak.update(|m| {
                        m.remove(&block_idx);
                    });
                    set_cards_completed.update(|c| *c += 1);
                } else {
                    // First "knew it" — insert at back of deck (half frequency)
                    set_knew_streak.update(|m| {
                        m.insert(block_idx, streak);
                    });
                    current_deck.push_back(block_idx);
                }
            }
        }

        if current_deck.is_empty() {
            set_is_open.set(false);
        } else {
            set_deck.set(current_deck);
            set_revealed.set(false);
        }
    };

    return view! {
        {move || {
            if !is_open.get() {
                return None;
            }
            let pinned = blocks.get();
            let current_deck = deck.get();
            let remaining = current_deck.len();
            if remaining == 0 {
                return None;
            }
            let block_idx = current_deck[0];
            let Some(block) = pinned.get(block_idx) else {
                set_is_open.set(false);
                return None;
            };
            let heading = format_block_heading(block);
            let content_html = block.preview.clone();
            let is_revealed = revealed.get();
            let score = scores.get().get(&block.label).cloned().unwrap_or_default();
            let completed = cards_completed.get();
            let stats = format!(
                "{} remaining \u{b7} {} done \u{b7} {} \u{2713} / {} ~ / {} \u{2717}",
                remaining,
                completed,
                score.right,
                score.unsure,
                score.wrong,
            );
            Some(
                // Stale index (blocks changed externally) — close study mode
                view! {
                    <div class="study-overlay" on:click=move |_| set_is_open.set(false)>
                        <div
                            class="study-modal"
                            on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                        >
                            // Header
                            <div class="study-modal-header">
                                <span class="study-stats">{stats}</span>
                                <button
                                    class="study-close"
                                    on:click=move |_| set_is_open.set(false)
                                >
                                    "\u{2715}"
                                </button>
                            </div>

                            // Front: block kind + title
                            <div class="study-front">
                                <span class="study-kind">{heading}</span>
                            </div>

                            // Back: content (blurred until revealed)
                            <div
                                class="study-back"
                                class:revealed=is_revealed
                                inner_html=content_html
                            />

                            // Actions
                            <div class="study-actions">
                                {if !is_revealed {
                                    view! {
                                        <button
                                            class="study-reveal-btn"
                                            on:click=move |_| set_revealed.set(true)
                                        >
                                            "reveal"
                                        </button>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <div class="study-rating">
                                            <button
                                                class="study-btn wrong"
                                                on:click=move |_| handle_answer(0)
                                            >
                                                "didn\u{2019}t know"
                                            </button>
                                            <button
                                                class="study-btn unsure"
                                                on:click=move |_| handle_answer(1)
                                            >
                                                "unsure"
                                            </button>
                                            <button
                                                class="study-btn right"
                                                on:click=move |_| handle_answer(2)
                                            >
                                                "knew it"
                                            </button>
                                        </div>
                                    }
                                        .into_any()
                                }}
                            </div>
                        </div>
                    </div>
                },
            )
        }}
    };
}
