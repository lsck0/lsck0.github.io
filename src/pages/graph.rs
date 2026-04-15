use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, WheelEvent};

use crate::models::post::POSTS;

// ============================================================
// Data structures
// ============================================================

struct GraphNode {
    slug: String,
    title: String,
    tags: Vec<String>,
    x: f64,
    y: f64,
    velocity_x: f64,
    velocity_y: f64,
}

#[derive(Clone, Copy, PartialEq)]
enum EdgeKind {
    Reference,
    Series,
}

struct GraphEdge {
    source: usize,
    target: usize,
    kind: EdgeKind,
}

struct Camera {
    offset_x: f64,
    offset_y: f64,
    zoom: f64,
}

impl Camera {
    fn screen_to_world(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        let world_x = (screen_x - self.offset_x) / self.zoom;
        let world_y = (screen_y - self.offset_y) / self.zoom;
        return (world_x, world_y);
    }
}

// ============================================================
// Tag coloring — hash-based with accessible contrast
// ============================================================

/// Generate a stable color from a tag name using hashing. Colors are chosen
/// to have sufficient contrast against both dark (#080c12) and light (#ffffff)
/// backgrounds. Uses golden angle (137.508°) to maximally spread hues.
fn tag_hue(tag: &str) -> u32 {
    let index = tag.bytes().fold(0u32, |accumulator, byte| {
        accumulator.wrapping_mul(31).wrapping_add(byte as u32)
    });
    return ((index as f64 * 137.508) % 360.0) as u32;
}

fn tag_color(tag: &str) -> String {
    let hue = tag_hue(tag);
    return format!("hsl({hue}, 70%, 55%)");
}

fn tag_color_transparent(tag: &str, alpha: f64) -> String {
    let hue = tag_hue(tag);
    return format!("hsla({hue}, 70%, 55%, {alpha})");
}

// ============================================================
// Graph construction
// ============================================================

fn build_graph() -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let nodes: Vec<GraphNode> = POSTS
        .iter()
        .enumerate()
        .map(|(index, post)| {
            let tags: Vec<String> = post.tags().into_iter().map(|tag| tag.to_string()).collect();
            let angle = (index as f64) * 2.0 * std::f64::consts::PI / (POSTS.len() as f64);
            let radius = 200.0;
            GraphNode {
                slug: post.slug().to_string(),
                title: strip_latex(post.title()),
                tags,
                x: radius * angle.cos(),
                y: radius * angle.sin(),
                velocity_x: 0.0,
                velocity_y: 0.0,
            }
        })
        .collect();

    let mut edges: Vec<GraphEdge> = Vec::new();

    for (source_index, post) in POSTS.iter().enumerate() {
        for linked_slug in post.internal_links() {
            if let Some(target_index) = POSTS.iter().position(|post| post.slug() == *linked_slug) {
                edges.push(GraphEdge {
                    source: source_index,
                    target: target_index,
                    kind: EdgeKind::Reference,
                });
            }
        }
    }

    let mut series_groups: HashMap<&str, Vec<(usize, u32)>> = HashMap::new();
    for (index, post) in POSTS.iter().enumerate() {
        if let Some(series_name) = post.series() {
            let order = post.series_order().unwrap_or(0);
            series_groups.entry(series_name).or_default().push((index, order));
        }
    }
    for (_name, mut members) in series_groups {
        members.sort_by_key(|(_, order)| *order);
        for window in members.windows(2) {
            edges.push(GraphEdge {
                source: window[0].0,
                target: window[1].0,
                kind: EdgeKind::Series,
            });
        }
    }

    return (nodes, edges);
}

// ============================================================
// Force simulation
// ============================================================

const REPULSION_STRENGTH: f64 = 5000.0;
const ATTRACTION_STRENGTH: f64 = 0.005;
const IDEAL_EDGE_LENGTH: f64 = 150.0;
const DAMPING: f64 = 0.85;
const MIN_VELOCITY: f64 = 0.01;
const NODE_RADIUS: f64 = 8.0;
const TOPIC_COHESION: f64 = 0.0008;

fn simulation_step(nodes: &mut [GraphNode], edges: &[GraphEdge]) -> bool {
    let node_count = nodes.len();
    let mut forces: Vec<(f64, f64)> = vec![(0.0, 0.0); node_count];

    // Repulsion between all pairs
    for i in 0..node_count {
        for j in (i + 1)..node_count {
            let dx = nodes[i].x - nodes[j].x;
            let dy = nodes[i].y - nodes[j].y;
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt().max(1.0);
            let force = REPULSION_STRENGTH / dist_sq;
            let fx = force * dx / dist;
            let fy = force * dy / dist;
            forces[i].0 += fx;
            forces[i].1 += fy;
            forces[j].0 -= fx;
            forces[j].1 -= fy;
        }
    }

    // Attraction along edges
    for edge in edges {
        let dx = nodes[edge.target].x - nodes[edge.source].x;
        let dy = nodes[edge.target].y - nodes[edge.source].y;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        let force = ATTRACTION_STRENGTH * (dist - IDEAL_EDGE_LENGTH);
        let fx = force * dx / dist;
        let fy = force * dy / dist;
        forces[edge.source].0 += fx;
        forces[edge.source].1 += fy;
        forces[edge.target].0 -= fx;
        forces[edge.target].1 -= fy;
    }

    // Tag cohesion
    let mut tag_centroids: HashMap<&str, (f64, f64, f64)> = HashMap::new();
    for node in nodes.iter() {
        for tag in &node.tags {
            let entry = tag_centroids.entry(tag).or_insert((0.0, 0.0, 0.0));
            entry.0 += node.x;
            entry.1 += node.y;
            entry.2 += 1.0;
        }
    }
    for (index, node) in nodes.iter().enumerate() {
        for tag in &node.tags {
            if let Some(&(sum_x, sum_y, count)) = tag_centroids.get(tag.as_str())
                && count > 1.0
            {
                forces[index].0 += (sum_x / count - node.x) * TOPIC_COHESION;
                forces[index].1 += (sum_y / count - node.y) * TOPIC_COHESION;
            }
        }
    }

    // Centering force
    for i in 0..node_count {
        forces[i].0 -= nodes[i].x * 0.001;
        forces[i].1 -= nodes[i].y * 0.001;
    }

    // Apply forces
    let mut has_movement = false;
    for (index, node) in nodes.iter_mut().enumerate() {
        node.velocity_x = (node.velocity_x + forces[index].0) * DAMPING;
        node.velocity_y = (node.velocity_y + forces[index].1) * DAMPING;
        if node.velocity_x.abs() > MIN_VELOCITY || node.velocity_y.abs() > MIN_VELOCITY {
            has_movement = true;
        }
        node.x += node.velocity_x;
        node.y += node.velocity_y;
    }

    return has_movement;
}

// ============================================================
// Convex hull (Andrew's monotone chain)
// ============================================================

fn convex_hull(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut sorted: Vec<(f64, f64)> = points.to_vec();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));

    let mut hull: Vec<(f64, f64)> = Vec::new();

    for &point in &sorted {
        while hull.len() >= 2 {
            let second_last = hull[hull.len() - 2];
            let last = hull[hull.len() - 1];
            if cross(second_last, last, point) <= 0.0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(point);
    }

    let lower_len = hull.len();
    for &point in sorted.iter().rev() {
        while hull.len() > lower_len {
            let second_last = hull[hull.len() - 2];
            let last = hull[hull.len() - 1];
            if cross(second_last, last, point) <= 0.0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(point);
    }

    hull.pop();
    return hull;
}

fn cross(origin: (f64, f64), first: (f64, f64), second: (f64, f64)) -> f64 {
    return (first.0 - origin.0) * (second.1 - origin.1) - (first.1 - origin.1) * (second.0 - origin.0);
}

fn expand_hull(hull: &[(f64, f64)], padding: f64) -> Vec<(f64, f64)> {
    if hull.is_empty() {
        return vec![];
    }
    let center_x: f64 = hull.iter().map(|point| point.0).sum::<f64>() / hull.len() as f64;
    let center_y: f64 = hull.iter().map(|point| point.1).sum::<f64>() / hull.len() as f64;

    return hull
        .iter()
        .map(|&(x, y)| {
            let dx = x - center_x;
            let dy = y - center_y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            (x + dx / dist * padding, y + dy / dist * padding)
        })
        .collect();
}

// ============================================================
// Drawing
// ============================================================

const ARROWHEAD_LENGTH: f64 = 12.0;
const ARROWHEAD_ANGLE: f64 = 0.4;

fn draw_arrow(
    ctx: &CanvasRenderingContext2d,
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
    zoom: f64,
    node_radius: f64,
) {
    let dx = target_x - source_x;
    let dy = target_y - source_y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1.0 {
        return;
    }

    let unit_x = dx / dist;
    let unit_y = dy / dist;
    let radius = node_radius / zoom;
    let arrow_end_x = target_x - unit_x * radius;
    let arrow_end_y = target_y - unit_y * radius;
    let arrow_start_x = source_x + unit_x * radius;
    let arrow_start_y = source_y + unit_y * radius;

    ctx.begin_path();
    ctx.move_to(arrow_start_x, arrow_start_y);
    ctx.line_to(arrow_end_x, arrow_end_y);
    ctx.stroke();

    let arrowhead_size = ARROWHEAD_LENGTH / zoom;
    let angle = dy.atan2(dx);
    ctx.begin_path();
    ctx.move_to(arrow_end_x, arrow_end_y);
    ctx.line_to(
        arrow_end_x - arrowhead_size * (angle - ARROWHEAD_ANGLE).cos(),
        arrow_end_y - arrowhead_size * (angle - ARROWHEAD_ANGLE).sin(),
    );
    ctx.move_to(arrow_end_x, arrow_end_y);
    ctx.line_to(
        arrow_end_x - arrowhead_size * (angle + ARROWHEAD_ANGLE).cos(),
        arrow_end_y - arrowhead_size * (angle + ARROWHEAD_ANGLE).sin(),
    );
    ctx.stroke();
}

#[allow(clippy::too_many_arguments)]
fn draw_graph(
    ctx: &CanvasRenderingContext2d,
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    camera: &Camera,
    canvas_width: f64,
    canvas_height: f64,
    hovered_node: Option<usize>,
    is_dark_theme: bool,
    visible: &HashSet<String>,
) {
    let has_filter = !visible.is_empty() && visible.len() < nodes.len();
    let background = if is_dark_theme { "#080c12" } else { "#ffffff" };
    ctx.set_fill_style_str(background);
    ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height);

    ctx.save();
    let _ = ctx.translate(camera.offset_x, camera.offset_y);
    let _ = ctx.scale(camera.zoom, camera.zoom);

    // ---- Tag regions ----
    let mut tag_points: HashMap<&str, Vec<(f64, f64)>> = HashMap::new();
    for node in nodes {
        if has_filter && !visible.contains(&node.slug) {
            continue;
        }
        for tag in &node.tags {
            tag_points.entry(tag).or_default().push((node.x, node.y));
        }
    }

    for (tag, points) in &tag_points {
        if points.len() < 2 {
            continue;
        }
        let hull = convex_hull(points);
        let expanded = expand_hull(&hull, 40.0 / camera.zoom);
        if expanded.len() < 2 {
            continue;
        }

        let fill_alpha = if is_dark_theme { 0.04 } else { 0.06 };
        let border_alpha = if is_dark_theme { 0.12 } else { 0.15 };

        ctx.begin_path();
        ctx.move_to(expanded[0].0, expanded[0].1);
        for point in &expanded[1..] {
            ctx.line_to(point.0, point.1);
        }
        ctx.close_path();
        ctx.set_fill_style_str(&tag_color_transparent(tag, fill_alpha));
        ctx.fill();

        ctx.set_stroke_style_str(&tag_color_transparent(tag, border_alpha));
        ctx.set_line_width(1.5 / camera.zoom);
        ctx.set_line_dash(&js_sys::Array::of2(
            &(6.0 / camera.zoom).into(),
            &(4.0 / camera.zoom).into(),
        ))
        .ok();
        ctx.stroke();
        let _ = ctx.set_line_dash(&js_sys::Array::new());
    }

    // ---- Edges ----
    let hovered_edges: Vec<usize> = if let Some(hovered) = hovered_node {
        edges
            .iter()
            .enumerate()
            .filter(|(_, e)| e.source == hovered || e.target == hovered)
            .map(|(i, _)| i)
            .collect()
    } else {
        vec![]
    };

    let line_base = 1.5 / camera.zoom;

    for (i, edge) in edges.iter().enumerate() {
        let source = &nodes[edge.source];
        let target = &nodes[edge.target];

        if has_filter && (!visible.contains(&source.slug) || !visible.contains(&target.slug)) {
            continue;
        }

        let highlighted = hovered_edges.contains(&i);

        let (color, is_dashed, width_multiplier) = match edge.kind {
            EdgeKind::Reference => {
                let alpha = if highlighted { 0.8 } else { 0.35 };
                let edge_color = if is_dark_theme {
                    format!("rgba(139, 148, 158, {alpha})")
                } else {
                    format!("rgba(106, 115, 125, {alpha})")
                };
                (edge_color, false, if highlighted { 2.0 } else { 1.0 })
            }
            EdgeKind::Series => {
                let alpha = if highlighted { 0.9 } else { 0.5 };
                let edge_color = if is_dark_theme {
                    format!("rgba(225, 192, 105, {alpha})")
                } else {
                    format!("rgba(180, 145, 50, {alpha})")
                };
                (edge_color, true, if highlighted { 2.5 } else { 1.5 })
            }
        };

        ctx.set_stroke_style_str(&color);
        ctx.set_line_width(line_base * width_multiplier);

        if is_dashed {
            ctx.set_line_dash(&js_sys::Array::of2(
                &(8.0 / camera.zoom).into(),
                &(4.0 / camera.zoom).into(),
            ))
            .ok();
        }

        draw_arrow(ctx, source.x, source.y, target.x, target.y, camera.zoom, NODE_RADIUS);

        if is_dashed {
            let _ = ctx.set_line_dash(&js_sys::Array::new());
        }
    }

    // ---- Nodes ----
    for (index, node) in nodes.iter().enumerate() {
        if has_filter && !visible.contains(&node.slug) {
            continue;
        }
        let is_hovered = hovered_node == Some(index);
        let is_neighbor = if let Some(hovered) = hovered_node {
            edges.iter().any(|edge| {
                (edge.source == hovered && edge.target == index) || (edge.target == hovered && edge.source == index)
            })
        } else {
            false
        };

        let base_radius = NODE_RADIUS / camera.zoom;
        let radius = if is_hovered {
            base_radius * 1.6
        } else if is_neighbor {
            base_radius * 1.2
        } else {
            base_radius
        };

        let primary_tag = node.tags.first().map(|tag| tag.as_str()).unwrap_or("other");
        let color = tag_color(primary_tag);

        if is_hovered || is_neighbor {
            ctx.set_shadow_color(&color);
            ctx.set_shadow_blur(12.0 / camera.zoom);
        }

        ctx.begin_path();
        let _ = ctx.arc(node.x, node.y, radius, 0.0, 2.0 * std::f64::consts::PI);
        ctx.set_fill_style_str(&color);
        ctx.fill();

        if hovered_node.is_some() && !is_hovered && !is_neighbor {
            ctx.set_global_alpha(0.4);
        }

        ctx.set_shadow_color("transparent");
        ctx.set_shadow_blur(0.0);
        ctx.set_global_alpha(1.0);

        let font_size = if is_hovered { 13.0 } else { 10.0 } / camera.zoom;
        ctx.set_font(&format!("{}px monospace", font_size));

        if hovered_node.is_some() && !is_hovered && !is_neighbor {
            let muted = if is_dark_theme {
                "rgba(201, 209, 217, 0.3)"
            } else {
                "rgba(36, 41, 46, 0.3)"
            };
            ctx.set_fill_style_str(muted);
        } else {
            let text = if is_dark_theme { "#c9d1d9" } else { "#24292e" };
            ctx.set_fill_style_str(text);
        }

        ctx.set_text_align("center");
        let _ = ctx.fill_text(&node.title, node.x, node.y - radius - 4.0 / camera.zoom);
    }

    ctx.restore();
}

fn find_node_at(nodes: &[GraphNode], camera: &Camera, screen_x: f64, screen_y: f64) -> Option<usize> {
    let (world_x, world_y) = camera.screen_to_world(screen_x, screen_y);
    let hit_radius = NODE_RADIUS * 2.0 / camera.zoom;
    for (index, node) in nodes.iter().enumerate() {
        let dx = node.x - world_x;
        let dy = node.y - world_y;
        if dx * dx + dy * dy < hit_radius * hit_radius {
            return Some(index);
        }
    }
    return None;
}

// ============================================================
// Component
// ============================================================

#[component]
pub fn GraphView(#[prop(into)] visible_slugs: Signal<Vec<String>>) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    Effect::new(move |_: Option<()>| {
        let Some(canvas_element) = canvas_ref.get() else {
            return;
        };
        let canvas: HtmlCanvasElement = canvas_element;

        let Some(window) = web_sys::window() else { return };
        let dpr = window.device_pixel_ratio();

        let rect = canvas.get_bounding_client_rect();
        let display_width = rect.width();
        let display_height = rect.height();
        canvas.set_width((display_width * dpr) as u32);
        canvas.set_height((display_height * dpr) as u32);

        let Ok(Some(ctx_obj)) = canvas.get_context("2d") else {
            return;
        };
        let Ok(ctx) = ctx_obj.dyn_into::<CanvasRenderingContext2d>() else {
            return;
        };
        let _ = ctx.scale(dpr, dpr);

        let (mut nodes, edges) = build_graph();

        // run partial simulation so the layout is roughly formed but still settles visually
        for _ in 0..150 {
            simulation_step(&mut nodes, &edges);
        }

        let nodes = Rc::new(RefCell::new(nodes));
        let edges = Rc::new(edges);
        let camera = Rc::new(RefCell::new(Camera {
            offset_x: display_width / 2.0,
            offset_y: display_height / 2.0,
            zoom: 1.0,
        }));
        let hovered_node: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
        let is_dragging = Rc::new(RefCell::new(false));
        let drag_start = Rc::new(RefCell::new((0.0, 0.0)));
        let mouse_down_origin = Rc::new(RefCell::new((0.0, 0.0)));
        let dragged_node: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
        let visible_set: Rc<RefCell<HashSet<String>>> = Rc::new(RefCell::new(HashSet::new()));

        {
            let slugs = visible_slugs.get_untracked();
            *visible_set.borrow_mut() = slugs.into_iter().collect();
        }

        let is_dark = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.document_element())
            .map(|element| element.get_attribute("data-theme").is_some_and(|theme| theme == "dark"))
            .unwrap_or(true);

        // ---- Animation loop ----
        let anim_ctx = ctx.clone();
        let anim_nodes = Rc::clone(&nodes);
        let anim_edges = Rc::clone(&edges);
        let anim_camera = Rc::clone(&camera);
        let anim_hovered = Rc::clone(&hovered_node);
        let anim_dragged = Rc::clone(&dragged_node);
        let anim_visible = Rc::clone(&visible_set);

        type AnimCallback = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;
        let animation: AnimCallback = Rc::new(RefCell::new(None));
        let animation_ref = Rc::clone(&animation);
        let is_animating = Rc::new(RefCell::new(false));
        let is_animating_ref = Rc::clone(&is_animating);

        *animation.borrow_mut() = Some(Closure::new(move || {
            let mut nodes_ref = anim_nodes.borrow_mut();
            let camera_ref = anim_camera.borrow();
            let hovered_ref = *anim_hovered.borrow();
            let dragged_ref = *anim_dragged.borrow();
            let visible_ref = anim_visible.borrow();

            let has_movement = if let Some(dragged_index) = dragged_ref {
                let (saved_x, saved_y) = (nodes_ref[dragged_index].x, nodes_ref[dragged_index].y);
                let moved = simulation_step(&mut nodes_ref, &anim_edges);
                nodes_ref[dragged_index].x = saved_x;
                nodes_ref[dragged_index].y = saved_y;
                nodes_ref[dragged_index].velocity_x = 0.0;
                nodes_ref[dragged_index].velocity_y = 0.0;
                moved
            } else {
                simulation_step(&mut nodes_ref, &anim_edges)
            };

            draw_graph(
                &anim_ctx,
                &nodes_ref,
                &anim_edges,
                &camera_ref,
                display_width,
                display_height,
                hovered_ref,
                is_dark,
                &visible_ref,
            );

            if has_movement || dragged_ref.is_some() {
                if let Some(closure) = animation_ref.borrow().as_ref()
                    && let Some(w) = web_sys::window()
                {
                    let _ = w.request_animation_frame(closure.as_ref().unchecked_ref());
                }
            } else {
                *is_animating_ref.borrow_mut() = false;
            }
        }));

        let restart_animation = {
            let animation = Rc::clone(&animation);
            let is_animating = Rc::clone(&is_animating);
            move || {
                if !*is_animating.borrow() {
                    *is_animating.borrow_mut() = true;
                    if let Some(closure) = animation.borrow().as_ref()
                        && let Some(w) = web_sys::window()
                    {
                        let _ = w.request_animation_frame(closure.as_ref().unchecked_ref());
                    }
                }
            }
        };

        restart_animation();

        // ---- Reactive filter update ----
        {
            let filter_visible = Rc::clone(&visible_set);
            let filter_restart = restart_animation.clone();
            Effect::new(move |_: Option<()>| {
                let slugs = visible_slugs.get();
                *filter_visible.borrow_mut() = slugs.into_iter().collect();
                filter_restart();
            });
        }

        // ---- Mouse move ----
        {
            let move_nodes = Rc::clone(&nodes);
            let move_camera = Rc::clone(&camera);
            let move_hovered = Rc::clone(&hovered_node);
            let move_dragging = Rc::clone(&is_dragging);
            let move_drag_start = Rc::clone(&drag_start);
            let move_dragged = Rc::clone(&dragged_node);
            let move_canvas = canvas.clone();
            let move_restart = restart_animation.clone();

            let closure = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
                let rect = move_canvas.get_bounding_client_rect();
                let screen_x = event.client_x() as f64 - rect.left();
                let screen_y = event.client_y() as f64 - rect.top();

                if *move_dragging.borrow() {
                    let (start_x, start_y) = *move_drag_start.borrow();
                    let delta_x = screen_x - start_x;
                    let delta_y = screen_y - start_y;

                    if let Some(node_index) = *move_dragged.borrow() {
                        let camera_ref = move_camera.borrow();
                        let (world_x, world_y) = camera_ref.screen_to_world(screen_x, screen_y);
                        drop(camera_ref);
                        let mut nodes_ref = move_nodes.borrow_mut();
                        nodes_ref[node_index].x = world_x;
                        nodes_ref[node_index].y = world_y;
                        nodes_ref[node_index].velocity_x = 0.0;
                        nodes_ref[node_index].velocity_y = 0.0;
                    } else {
                        let mut camera_ref = move_camera.borrow_mut();
                        camera_ref.offset_x += delta_x;
                        camera_ref.offset_y += delta_y;
                    }
                    *move_drag_start.borrow_mut() = (screen_x, screen_y);
                    move_restart();
                } else {
                    let camera_ref = move_camera.borrow();
                    let found = find_node_at(&move_nodes.borrow(), &camera_ref, screen_x, screen_y);
                    let old_hovered = *move_hovered.borrow();
                    *move_hovered.borrow_mut() = found;
                    if found != old_hovered {
                        move_restart();
                    }
                    let element: &web_sys::HtmlElement = move_canvas.unchecked_ref();
                    let _ = element
                        .style()
                        .set_property("cursor", if found.is_some() { "pointer" } else { "grab" });
                }
            });
            let _ = canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        // ---- Mouse down ----
        {
            let down_nodes = Rc::clone(&nodes);
            let down_camera = Rc::clone(&camera);
            let down_dragging = Rc::clone(&is_dragging);
            let down_drag_start = Rc::clone(&drag_start);
            let down_origin = Rc::clone(&mouse_down_origin);
            let down_dragged = Rc::clone(&dragged_node);
            let down_canvas = canvas.clone();
            let down_restart = restart_animation.clone();

            let closure = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
                let rect = down_canvas.get_bounding_client_rect();
                let screen_x = event.client_x() as f64 - rect.left();
                let screen_y = event.client_y() as f64 - rect.top();

                *down_dragging.borrow_mut() = true;
                *down_drag_start.borrow_mut() = (screen_x, screen_y);
                *down_origin.borrow_mut() = (screen_x, screen_y);

                let camera_ref = down_camera.borrow();
                *down_dragged.borrow_mut() = find_node_at(&down_nodes.borrow(), &camera_ref, screen_x, screen_y);

                let element: &web_sys::HtmlElement = down_canvas.unchecked_ref();
                let _ = element.style().set_property("cursor", "grabbing");
                down_restart();
            });
            let _ = canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        // ---- Mouse up ----
        {
            let up_nodes = Rc::clone(&nodes);
            let up_camera = Rc::clone(&camera);
            let up_dragging = Rc::clone(&is_dragging);
            let up_dragged = Rc::clone(&dragged_node);
            let up_origin = Rc::clone(&mouse_down_origin);
            let up_canvas = canvas.clone();
            // Get navigate function outside the closure to avoid calling hooks in event handler
            let navigate = leptos_router::hooks::use_navigate();

            let closure = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
                let (origin_x, origin_y) = *up_origin.borrow();
                let rect = up_canvas.get_bounding_client_rect();
                let screen_x = event.client_x() as f64 - rect.left();
                let screen_y = event.client_y() as f64 - rect.top();
                let distance_moved = ((screen_x - origin_x).powi(2) + (screen_y - origin_y).powi(2)).sqrt();

                *up_dragging.borrow_mut() = false;
                *up_dragged.borrow_mut() = None;

                if distance_moved < 5.0 {
                    let camera_ref = up_camera.borrow();
                    if let Some(node_index) = find_node_at(&up_nodes.borrow(), &camera_ref, screen_x, screen_y) {
                        let slug = &up_nodes.borrow()[node_index].slug;
                        let href = format!("/blog/{slug}");
                        navigate(&href, Default::default());
                    }
                }

                let element: &web_sys::HtmlElement = up_canvas.unchecked_ref();
                let _ = element.style().set_property("cursor", "grab");
            });
            let _ = canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        // ---- Wheel (zoom) ----
        {
            let zoom_camera = Rc::clone(&camera);
            let zoom_canvas = canvas.clone();
            let zoom_restart = restart_animation.clone();

            let closure = Closure::<dyn FnMut(WheelEvent)>::new(move |event: WheelEvent| {
                event.prevent_default();
                let rect = zoom_canvas.get_bounding_client_rect();
                let mouse_x = event.client_x() as f64 - rect.left();
                let mouse_y = event.client_y() as f64 - rect.top();
                let factor = if event.delta_y() < 0.0 { 1.1 } else { 1.0 / 1.1 };

                let mut camera_ref = zoom_camera.borrow_mut();
                let new_zoom = (camera_ref.zoom * factor).clamp(0.1, 5.0);
                let scale = new_zoom / camera_ref.zoom;
                camera_ref.offset_x = mouse_x - (mouse_x - camera_ref.offset_x) * scale;
                camera_ref.offset_y = mouse_y - (mouse_y - camera_ref.offset_y) * scale;
                camera_ref.zoom = new_zoom;
                zoom_restart();
            });
            let opts = web_sys::AddEventListenerOptions::new();
            opts.set_passive(false);
            let _ = canvas.add_event_listener_with_callback_and_add_event_listener_options(
                "wheel",
                closure.as_ref().unchecked_ref(),
                &opts,
            );
            closure.forget();
        }

        // ---- Cleanup ----
        let cleanup_animation = Rc::clone(&animation);
        let cleanup_fn: js_sys::Function = Closure::<dyn FnMut()>::new(move || {
            *cleanup_animation.borrow_mut() = None;
        })
        .into_js_value()
        .unchecked_into();
        on_cleanup(move || {
            let _ = cleanup_fn.call0(&wasm_bindgen::JsValue::NULL);
        });
    });

    return view! {
        <div class="graph-container">
            <div class="graph-controls">
                <span class="graph-hint">"drag to pan · scroll to zoom · click to open"</span>
            </div>
            <canvas node_ref=canvas_ref class="graph-canvas" />
        </div>
    };
}

// ============================================================
// Helpers
// ============================================================

/// Strip `$...$` delimiters and common LaTeX commands for canvas text rendering.
fn strip_latex(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            // consume until closing $
            let mut inner = String::new();
            for inner_ch in chars.by_ref() {
                if inner_ch == '$' {
                    break;
                }
                inner.push(inner_ch);
            }
            // strip \mathbb{X} → X, \command → ""
            result.push_str(&strip_latex_commands(&inner));
        } else {
            result.push(ch);
        }
    }
    return result;
}

fn strip_latex_commands(latex: &str) -> String {
    let mut result = String::new();
    let mut chars = latex.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            // consume command name
            let mut cmd = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_ascii_alphabetic() {
                    cmd.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            // if followed by {content}, extract content
            if chars.peek() == Some(&'{') {
                chars.next(); // skip {
                let mut depth = 1;
                let mut content = String::new();
                for brace_ch in chars.by_ref() {
                    if brace_ch == '{' {
                        depth += 1;
                    } else if brace_ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    content.push(brace_ch);
                }
                result.push_str(&strip_latex_commands(&content));
            }
        } else if ch == '{' || ch == '}' {
            // skip stray braces
        } else {
            result.push(ch);
        }
    }
    return result;
}
