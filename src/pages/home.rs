#![allow(clippy::needless_return)]

use std::{cell::RefCell, f64::consts::PI, rc::Rc, time::Duration};

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::A;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::components::footer::Footer;

const MESSAGES: &[&str] = &[
    "welcome to my homepage",
    "i am a mathematician and software engineer",
    "check out my projects and publications",
];

const TYPE_SPEED: u64 = 80;
const DELETE_SPEED: u64 = 40;
const PAUSE_TIME: u64 = 2000;

#[component]
pub fn Home() -> impl IntoView {
    let (display_text, set_display_text) = signal(String::new());

    // Particle animation
    let anim_running = RwSignal::new(true);
    // Terminal typing effect
    let typing_running = RwSignal::new(true);

    Effect::new(move |_: Option<()>| {
        anim_running.set(true);
        request_animation_frame(move || {
            if let Some(el) = document().get_element_by_id("simplicial-bg")
                && let Ok(canvas) = el.dyn_into::<HtmlCanvasElement>()
            {
                run_particle_bg(canvas, anim_running);
            }
        });
    });

    on_cleanup(move || {
        anim_running.set(false);
        typing_running.set(false);
    });

    Effect::new(move |_: Option<()>| {
        typing_running.set(true);
        let msg_index = StoredValue::new(0usize);
        let char_index = StoredValue::new(0usize);
        let is_deleting = StoredValue::new(false);

        fn schedule_tick(
            running: RwSignal<bool>,
            msg_index: StoredValue<usize>,
            char_index: StoredValue<usize>,
            is_deleting: StoredValue<bool>,
            set_display_text: WriteSignal<String>,
        ) {
            if !running.get_untracked() {
                return;
            }

            let current_msg = MESSAGES[msg_index.get_value()];

            if is_deleting.get_value() {
                let ci = char_index.get_value().saturating_sub(1);
                char_index.set_value(ci);
                set_display_text.set(current_msg[..ci].to_string());
                if ci == 0 {
                    is_deleting.set_value(false);
                    msg_index.set_value((msg_index.get_value() + 1) % MESSAGES.len());
                    set_timeout(
                        move || schedule_tick(running, msg_index, char_index, is_deleting, set_display_text),
                        Duration::from_millis(TYPE_SPEED),
                    );
                } else {
                    set_timeout(
                        move || schedule_tick(running, msg_index, char_index, is_deleting, set_display_text),
                        Duration::from_millis(DELETE_SPEED),
                    );
                }
            } else {
                let ci = char_index.get_value() + 1;
                char_index.set_value(ci);
                set_display_text.set(current_msg[..ci].to_string());
                if ci >= current_msg.len() {
                    is_deleting.set_value(true);
                    set_timeout(
                        move || schedule_tick(running, msg_index, char_index, is_deleting, set_display_text),
                        Duration::from_millis(PAUSE_TIME),
                    );
                } else {
                    set_timeout(
                        move || schedule_tick(running, msg_index, char_index, is_deleting, set_display_text),
                        Duration::from_millis(TYPE_SPEED),
                    );
                }
            }
        }

        schedule_tick(typing_running, msg_index, char_index, is_deleting, set_display_text);
    });

    return view! {
        <Title text="\u{03bb} lsck0" />
        <canvas id="simplicial-bg" class="simplicial-bg"></canvas>
        <main class="home">
            <div class="home-content">
                <h1 class="home-logo">
                    {"\u{03bb}"} <span class="home-handle">" Luca Sandrock"</span>
                </h1>
                <div class="terminal-line">
                    <span class="terminal-prompt">">"</span>
                    <span class="terminal-text">{display_text}</span>
                    <span class="terminal-cursor">"\u{2588}"</span>
                </div>
                <nav class="home-links">
                    <A href="/about">"about"</A>
                    <A href="/blog">"blog"</A>
                    <A href="/projects">"projects"</A>
                    <A href="/publications">"publications"</A>
                </nav>
            </div>
        </main>
        <Footer />
    };
}

// ---- Particle background animation ----

type AnimationCallback = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

const EDGE_DIST: f64 = 180.0;
const TRI_DIST: f64 = 145.0;
const SPEED: f64 = 0.3;

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

fn particle_distance(a: &Particle, b: &Particle) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn accent_rgb() -> &'static str {
    let dark = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .and_then(|e| e.get_attribute("data-theme"))
        .is_some_and(|t| t == "dark");
    if dark { "121, 184, 255" } else { "74, 144, 217" }
}

fn run_particle_bg(canvas: HtmlCanvasElement, running: RwSignal<bool>) {
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d").unwrap().unwrap().dyn_into().unwrap();

    let w = canvas.offset_width().max(1) as u32;
    let h = canvas.offset_height().max(1) as u32;
    canvas.set_width(w);
    canvas.set_height(h);

    let area = (w as f64) * (h as f64);
    let num = ((area / 18000.0).round() as usize).max(15);

    let mut particles = Vec::with_capacity(num);
    for _ in 0..num {
        particles.push(Particle {
            x: js_sys::Math::random() * w as f64,
            y: js_sys::Math::random() * h as f64,
            vx: (js_sys::Math::random() - 0.5) * SPEED,
            vy: (js_sys::Math::random() - 0.5) * SPEED,
        });
    }

    let state = Rc::new(RefCell::new(particles));

    let f: AnimationCallback = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if !running.get_untracked() {
            return;
        }

        let cw = canvas.offset_width().max(1) as f64;
        let ch = canvas.offset_height().max(1) as f64;
        if canvas.width() != cw as u32 || canvas.height() != ch as u32 {
            canvas.set_width(cw as u32);
            canvas.set_height(ch as u32);
        }

        let rgb = accent_rgb();
        let mut pts = state.borrow_mut();

        for p in pts.iter_mut() {
            p.x += p.vx;
            p.y += p.vy;
            if p.x < 0.0 || p.x > cw {
                p.vx *= -1.0;
            }
            if p.y < 0.0 || p.y > ch {
                p.vy *= -1.0;
            }
            p.x = p.x.clamp(0.0, cw);
            p.y = p.y.clamp(0.0, ch);
        }

        ctx.clear_rect(0.0, 0.0, cw, ch);

        // 2-simplices (filled triangles)
        for i in 0..pts.len() {
            for j in (i + 1)..pts.len() {
                let dij = particle_distance(&pts[i], &pts[j]);
                if dij > TRI_DIST {
                    continue;
                }
                for k in (j + 1)..pts.len() {
                    let dik = particle_distance(&pts[i], &pts[k]);
                    if dik > TRI_DIST {
                        continue;
                    }
                    let djk = particle_distance(&pts[j], &pts[k]);
                    if djk > TRI_DIST {
                        continue;
                    }
                    let max_d = dij.max(dik).max(djk);
                    let alpha = 0.12 * (1.0 - max_d / TRI_DIST);
                    ctx.begin_path();
                    ctx.move_to(pts[i].x, pts[i].y);
                    ctx.line_to(pts[j].x, pts[j].y);
                    ctx.line_to(pts[k].x, pts[k].y);
                    ctx.close_path();
                    let s = format!("rgba({rgb}, {alpha})");
                    ctx.set_fill_style_str(&s);
                    ctx.fill();
                }
            }
        }

        // 1-simplices (edges)
        ctx.set_line_width(1.2);
        for i in 0..pts.len() {
            for j in (i + 1)..pts.len() {
                let d = particle_distance(&pts[i], &pts[j]);
                if d < EDGE_DIST {
                    let alpha = 0.3 * (1.0 - d / EDGE_DIST);
                    ctx.begin_path();
                    ctx.move_to(pts[i].x, pts[i].y);
                    ctx.line_to(pts[j].x, pts[j].y);
                    let s = format!("rgba({rgb}, {alpha})");
                    ctx.set_stroke_style_str(&s);
                    ctx.stroke();
                }
            }
        }

        // 0-simplices (vertices)
        let neighbor_counts: Vec<usize> = (0..pts.len())
            .map(|i| {
                (0..pts.len())
                    .filter(|&j| j != i && particle_distance(&pts[i], &pts[j]) < EDGE_DIST)
                    .count()
            })
            .collect();

        for (i, &count) in neighbor_counts.iter().enumerate() {
            let alpha = 0.15 + 0.08 * (count.min(5) as f64);
            ctx.begin_path();
            let _ = ctx.arc(pts[i].x, pts[i].y, 2.0, 0.0, PI * 2.0);
            let s = format!("rgba({rgb}, {alpha})");
            ctx.set_fill_style_str(&s);
            ctx.fill();
        }

        drop(pts);

        if let Some(window) = web_sys::window() {
            let _ = window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut()>));

    if let Some(window) = web_sys::window() {
        let _ = window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }
}
