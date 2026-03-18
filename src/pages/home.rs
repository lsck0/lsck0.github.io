use std::{cell::RefCell, f64::consts::PI, rc::Rc, time::Duration};

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::A;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{components::footer::Footer, models::meta::META};

const HOME_CONTENT: &str = include_str!("../../content/home.md");

fn messages() -> Vec<&'static str> {
    return HOME_CONTENT.lines().filter(|line| !line.is_empty()).collect();
}

const TYPE_SPEED: u64 = 65;
const DELETE_SPEED: u64 = 35;
const PAUSE_TIME: u64 = 2400;

// ============================================================
// Component
// ============================================================

#[component]
pub fn HomePage() -> impl IntoView {
    let (display_text, set_display_text) = signal(String::new());
    let anim_running = RwSignal::new(true);
    let typing_running = RwSignal::new(true);
    let anim_initialized = std::cell::Cell::new(false);

    Effect::new(move |_: Option<()>| {
        if anim_initialized.get() {
            return;
        }
        anim_initialized.set(true);

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

            let msgs = messages();
            if msgs.is_empty() {
                return;
            }
            let current_msg = msgs[msg_index.get_value() % msgs.len()];
            let char_count = current_msg.chars().count();

            if is_deleting.get_value() {
                let current_char = char_index.get_value().saturating_sub(1);
                char_index.set_value(current_char);
                let text: String = current_msg.chars().take(current_char).collect();
                set_display_text.set(text);
                if current_char == 0 {
                    is_deleting.set_value(false);
                    msg_index.set_value((msg_index.get_value() + 1) % msgs.len());
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
                let current_char = char_index.get_value() + 1;
                char_index.set_value(current_char);
                let text: String = current_msg.chars().take(current_char).collect();
                set_display_text.set(text);
                if current_char >= char_count {
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
        <Title text=META.page_title("home") />
        <Meta name="description" content=META.page("home").map(|page| page.description).unwrap_or("") />
        <canvas id="simplicial-bg" class="simplicial-bg"></canvas>
        <main class="home">
            <div class="home-content">
                <div class="home-hero">
                    <div class="home-lambda">{"\u{03bb}"}</div>
                    <h1 class="home-title">"/dev/lsck0"</h1>
                    <div class="home-subtitle">"mathematician and software engineer"</div>
                </div>

                <div class="terminal-line">
                    <span class="terminal-prompt">{"\u{276f}"}</span>
                    <span class="terminal-text">{display_text}</span>
                    <span class="terminal-cursor">{"\u{258f}"}</span>
                </div>

                <nav class="home-links">
                    <A href="/about" attr:class="home-link">
                        "about"
                    </A>
                    <A href="/blog" attr:class="home-link">
                        "blog"
                    </A>
                    <A href="/projects" attr:class="home-link">
                        "projects"
                    </A>
                    <A href="/publications" attr:class="home-link">
                        "publications"
                    </A>
                </nav>

            </div>
        </main>
        <Footer />
    };
}

// ============================================================
// Particle background animation
// ============================================================

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

fn particle_distance(first: &Particle, second: &Particle) -> f64 {
    let dx = first.x - second.x;
    let dy = first.y - second.y;
    return (dx * dx + dy * dy).sqrt();
}

fn accent_rgb() -> &'static str {
    let is_dark_theme = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
        .and_then(|element| element.get_attribute("data-theme"))
        .is_some_and(|theme| theme == "dark");
    if is_dark_theme { "121, 184, 255" } else { "74, 144, 217" }
}

fn run_particle_bg(canvas: HtmlCanvasElement, running: RwSignal<bool>) {
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d").unwrap().unwrap().dyn_into().unwrap();

    let width = canvas.offset_width().max(1) as u32;
    let height = canvas.offset_height().max(1) as u32;
    canvas.set_width(width);
    canvas.set_height(height);

    let area = (width as f64) * (height as f64);
    let particle_count = ((area / 14000.0).round() as usize).max(15);

    let mut particles = Vec::with_capacity(particle_count);
    for _ in 0..particle_count {
        particles.push(Particle {
            x: js_sys::Math::random() * width as f64,
            y: js_sys::Math::random() * height as f64,
            vx: (js_sys::Math::random() - 0.5) * SPEED,
            vy: (js_sys::Math::random() - 0.5) * SPEED,
        });
    }

    let particle_state = Rc::new(RefCell::new(particles));
    let previous_size = Rc::new(RefCell::new((width as f64, height as f64)));

    let animation_closure: AnimationCallback = Rc::new(RefCell::new(None));
    let animation_handle = animation_closure.clone();

    *animation_handle.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if !running.get_untracked() {
            return;
        }

        let canvas_width = canvas.offset_width().max(1) as f64;
        let canvas_height = canvas.offset_height().max(1) as f64;
        if canvas.width() != canvas_width as u32 || canvas.height() != canvas_height as u32 {
            canvas.set_width(canvas_width as u32);
            canvas.set_height(canvas_height as u32);

            let (old_width, old_height) = *previous_size.borrow();
            let mut particles = particle_state.borrow_mut();

            if old_width > 0.0 && old_height > 0.0 {
                let scale_x = canvas_width / old_width;
                let scale_y = canvas_height / old_height;
                for particle in particles.iter_mut() {
                    particle.x *= scale_x;
                    particle.y *= scale_y;
                }
            }

            let target_count = ((canvas_width * canvas_height / 14000.0).round() as usize).max(15);
            while particles.len() > target_count {
                particles.pop();
            }
            while particles.len() < target_count {
                particles.push(Particle {
                    x: js_sys::Math::random() * canvas_width,
                    y: js_sys::Math::random() * canvas_height,
                    vx: (js_sys::Math::random() - 0.5) * SPEED,
                    vy: (js_sys::Math::random() - 0.5) * SPEED,
                });
            }

            drop(particles);
            *previous_size.borrow_mut() = (canvas_width, canvas_height);
        }

        let rgb = accent_rgb();
        let mut particles = particle_state.borrow_mut();

        for particle in particles.iter_mut() {
            particle.x += particle.vx;
            particle.y += particle.vy;
            if particle.x < 0.0 || particle.x > canvas_width {
                particle.vx *= -1.0;
            }
            if particle.y < 0.0 || particle.y > canvas_height {
                particle.vy *= -1.0;
            }
            particle.x = particle.x.clamp(0.0, canvas_width);
            particle.y = particle.y.clamp(0.0, canvas_height);
        }

        ctx.clear_rect(0.0, 0.0, canvas_width, canvas_height);

        for i in 0..particles.len() {
            for j in (i + 1)..particles.len() {
                let dist_ij = particle_distance(&particles[i], &particles[j]);
                if dist_ij > TRI_DIST {
                    continue;
                }
                for k in (j + 1)..particles.len() {
                    let dist_ik = particle_distance(&particles[i], &particles[k]);
                    if dist_ik > TRI_DIST {
                        continue;
                    }
                    let dist_jk = particle_distance(&particles[j], &particles[k]);
                    if dist_jk > TRI_DIST {
                        continue;
                    }
                    let max_dist = dist_ij.max(dist_ik).max(dist_jk);
                    let alpha = 0.12 * (1.0 - max_dist / TRI_DIST);
                    ctx.begin_path();
                    ctx.move_to(particles[i].x, particles[i].y);
                    ctx.line_to(particles[j].x, particles[j].y);
                    ctx.line_to(particles[k].x, particles[k].y);
                    ctx.close_path();
                    ctx.set_fill_style_str(&format!("rgba({rgb}, {alpha})"));
                    ctx.fill();
                }
            }
        }

        ctx.set_line_width(1.2);
        for i in 0..particles.len() {
            for j in (i + 1)..particles.len() {
                let dist = particle_distance(&particles[i], &particles[j]);
                if dist < EDGE_DIST {
                    let alpha = 0.3 * (1.0 - dist / EDGE_DIST);
                    ctx.begin_path();
                    ctx.move_to(particles[i].x, particles[i].y);
                    ctx.line_to(particles[j].x, particles[j].y);
                    ctx.set_stroke_style_str(&format!("rgba({rgb}, {alpha})"));
                    ctx.stroke();
                }
            }
        }

        let neighbor_counts: Vec<usize> = (0..particles.len())
            .map(|i| {
                (0..particles.len())
                    .filter(|&j| j != i && particle_distance(&particles[i], &particles[j]) < EDGE_DIST)
                    .count()
            })
            .collect();

        for (i, &count) in neighbor_counts.iter().enumerate() {
            let alpha = 0.15 + 0.08 * (count.min(5) as f64);
            ctx.begin_path();
            let _ = ctx.arc(particles[i].x, particles[i].y, 2.0, 0.0, PI * 2.0);
            ctx.set_fill_style_str(&format!("rgba({rgb}, {alpha})"));
            ctx.fill();
        }

        drop(particles);

        if let Some(window) = web_sys::window() {
            let _ =
                window.request_animation_frame(animation_closure.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut()>));

    if let Some(window) = web_sys::window() {
        let _ = window.request_animation_frame(animation_handle.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }
}
