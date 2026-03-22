use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};

use super::SidebarState;
use crate::{
    components::listing::{ProjectStatus, render_segments, segments_has_scrambled, segments_plain_value},
    models::post::{POSTS, Post},
    pages::{projects::PROJECTS, publications::PUBLICATIONS},
};

// ============================================================
// Post tree data structures
// ============================================================

struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    posts: Vec<&'static Post>,
}

impl TreeNode {
    fn new() -> Self {
        return Self {
            children: BTreeMap::new(),
            posts: Vec::new(),
        };
    }
}

enum TreeItem {
    Folder {
        name: String,
        depth: usize,
        path: String,
    },
    PostEntry {
        post: &'static Post,
        depth: usize,
        parent: String,
    },
}

// ============================================================
// Sidebar component
// ============================================================

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<SidebarState>().expect("SidebarState context");
    let location = use_location();

    let tree = build_post_tree();
    let items = flatten_tree(&tree, 1, "");

    let close_mobile = move || {
        state.is_mobile_open.set(false);
    };

    return view! {
        <nav class="sidebar" class:open=move || state.is_mobile_open.get()>

            // ============================================================
            // About
            // ============================================================

            <A
                href="/about"
                attr:class=move || {
                    let path = location.pathname.get();
                    if path == "/about" { "sidebar-item active" } else { "sidebar-item" }
                }
                on:click=move |_| close_mobile()
            >
                "about"
            </A>

            // ============================================================
            // Blog
            // ============================================================

            <div
                class=move || {
                    let path = location.pathname.get();
                    if path.starts_with("/blog") {
                        "sidebar-item sidebar-section active"
                    } else {
                        "sidebar-item sidebar-section"
                    }
                }
                on:click=move |_| {
                    state.is_blog_open.update(|open| *open = !*open);
                }
            >
                <span class="folder-icon">
                    {move || if state.is_blog_open.get() { "\u{25be}" } else { "\u{25b8}" }}
                </span>
                " blog"
            </div>

            <ul
                class="file-tree"
                style:display=move || if state.is_blog_open.get() { "block" } else { "none" }
            >
                {items
                    .into_iter()
                    .map(|item| match item {
                        TreeItem::Folder { name, depth, path } => {
                            let path_for_click = path.clone();
                            let path_for_icon = path.clone();
                            let parent = parent_folder_of(&path);

                            view! {
                                <li
                                    class="tree-folder"
                                    style=move || {
                                        let padding = format!(
                                            "padding-left: {}rem",
                                            depth as f32 * 0.75,
                                        );
                                        if is_path_collapsed(
                                            &parent,
                                            &state.collapsed_folders.get(),
                                        ) {
                                            format!("{padding}; display: none")
                                        } else {
                                            padding
                                        }
                                    }
                                    on:click=move |_| {
                                        state
                                            .collapsed_folders
                                            .update(|set| {
                                                if !set.remove(&path_for_click) {
                                                    set.insert(path_for_click.clone());
                                                }
                                            });
                                    }
                                >
                                    <span class="folder-icon">
                                        {move || {
                                            if state.collapsed_folders.get().contains(&path_for_icon) {
                                                "\u{25b8}"
                                            } else {
                                                "\u{25be}"
                                            }
                                        }}
                                    </span>
                                    {format!(" {}/", name)}
                                </li>
                            }
                                .into_any()
                        }
                        TreeItem::PostEntry { post, depth, parent } => {
                            let slug = post.slug.to_string();
                            view! {
                                <li
                                    class=move || {
                                        let path = location.pathname.get();
                                        let expected = format!("/blog/{}", slug);
                                        if path == expected {
                                            "tree-post active"
                                        } else {
                                            "tree-post"
                                        }
                                    }
                                    style=move || {
                                        let padding = format!(
                                            "padding-left: {}rem",
                                            depth as f32 * 0.75,
                                        );
                                        if is_path_collapsed(
                                            &parent,
                                            &state.collapsed_folders.get(),
                                        ) {
                                            format!("{padding}; display: none")
                                        } else {
                                            padding
                                        }
                                    }
                                >
                                    <A
                                        href=format!("/blog/{}", post.slug)
                                        on:click=move |_| close_mobile()
                                    >
                                        {post.title()}
                                    </A>
                                </li>
                            }
                                .into_any()
                        }
                    })
                    .collect_view()}
            </ul>

            // ============================================================
            // Projects
            // ============================================================

            <div
                class=move || {
                    let path = location.pathname.get();
                    if path.starts_with("/projects") {
                        "sidebar-item sidebar-section active"
                    } else {
                        "sidebar-item sidebar-section"
                    }
                }
                on:click=move |_| {
                    state.is_projects_open.update(|open| *open = !*open);
                }
            >
                <span class="folder-icon">
                    {move || if state.is_projects_open.get() { "\u{25be}" } else { "\u{25b8}" }}
                </span>
                " "
                <A
                    href="/projects"
                    on:click=move |event| {
                        event.stop_propagation();
                        close_mobile();
                    }
                >
                    "projects"
                </A>
            </div>

            <ul
                class="file-tree"
                style:display=move || if state.is_projects_open.get() { "block" } else { "none" }
            >
                {
                    let private_projects: Vec<_> = PROJECTS
                        .iter()
                        .filter(|e| !e.is_professional())
                        .collect();
                    let professional_projects: Vec<_> = PROJECTS
                        .iter()
                        .filter(|e| e.is_professional())
                        .collect();
                    let private_status_views: Vec<_> = ProjectStatus::all()
                        .iter()
                        .filter(|&&status| {
                            private_projects.iter().any(|entry| entry.status == status)
                        })
                        .map(|&status| {
                            let status_id = format!("private-{}", status.id());
                            let status_id_for_click = status_id.clone();
                            let status_id_for_icon = status_id.clone();
                            let projects_for_status: Vec<_> = private_projects
                                .iter()
                                .filter(|entry| entry.status == status)
                                .copied()
                                .collect();
                            let project_views = projects_for_status
                                .into_iter()
                                .map(|entry| {
                                    let current_status_id = status_id.clone();
                                    let title_plain = segments_plain_value(entry.title);
                                    let href = format!("/projects#{}", title_plain);
                                    let label = if segments_has_scrambled(entry.title) {

                                        // Private projects grouped by status
                                        view! {
                                            <span class="teaser">{render_segments(entry.title)}</span>
                                        }
                                            .into_any()
                                    } else {
                                        view! { {title_plain} }.into_any()
                                    };
                                    view! {
                                        <li
                                            class="tree-post"
                                            style:padding-left="2.25rem"
                                            style:display=move || {
                                                if state
                                                    .collapsed_project_groups
                                                    .get()
                                                    .contains(&current_status_id)
                                                {
                                                    "none"
                                                } else {
                                                    "list-item"
                                                }
                                            }
                                        >
                                            <A href=href on:click=move |_| close_mobile()>
                                                {label}
                                            </A>
                                        </li>
                                    }
                                })
                                .collect_view();
                            view! {
                                <li
                                    class="tree-folder"
                                    style="padding-left: 1.5rem"
                                    on:click=move |_| {
                                        state
                                            .collapsed_project_groups
                                            .update(|set| {
                                                if !set.remove(&status_id_for_click) {
                                                    set.insert(status_id_for_click.clone());
                                                }
                                            });
                                    }
                                >
                                    <span class="folder-icon">
                                        {move || {
                                            if state
                                                .collapsed_project_groups
                                                .get()
                                                .contains(&status_id_for_icon)
                                            {
                                                "\u{25b8}"
                                            } else {
                                                "\u{25be}"
                                            }
                                        }}
                                    </span>
                                    {format!(" {}/", status.label())}
                                </li>
                                {project_views}
                            }
                        })
                        .collect();
                    let professional_views: Vec<_> = professional_projects
                        .clone()
                        .into_iter()
                        .map(|entry| {
                            let title_plain = segments_plain_value(entry.title);
                            let href = format!("/projects#{}", title_plain);
                            let label = if segments_has_scrambled(entry.title) {

                                // Professional projects flat list
                                view! { <span class="teaser">{render_segments(entry.title)}</span> }
                                    .into_any()
                            } else {
                                view! { {title_plain} }.into_any()
                            };
                            view! {
                                <li class="tree-post" style="padding-left: 1.5rem">
                                    <A href=href on:click=move |_| close_mobile()>
                                        {label}
                                    </A>
                                </li>
                            }
                        })
                        .collect();

                    view! {
                        <>
                            // Private Projects folder
                            {(!private_projects.is_empty())
                                .then(|| {
                                    view! {
                                        <li
                                            class="tree-folder"
                                            style="padding-left: 0.75rem"
                                            on:click=move |_| {
                                                state
                                                    .collapsed_project_groups
                                                    .update(|set| {
                                                        if !set.remove("private") {
                                                            set.insert("private".to_string());
                                                        }
                                                    });
                                            }
                                        >
                                            <span class="folder-icon">
                                                {move || {
                                                    if state.collapsed_project_groups.get().contains("private")
                                                    {
                                                        "\u{25b8}"
                                                    } else {
                                                        "\u{25be}"
                                                    }
                                                }}
                                            </span>
                                            " private/"
                                        </li>
                                        <div style:display=move || {
                                            if state.collapsed_project_groups.get().contains("private")
                                            {
                                                "none"
                                            } else {
                                                "block"
                                            }
                                        }>{private_status_views}</div>
                                    }
                                })} // Professional Projects folder
                            {(!professional_projects.is_empty())
                                .then(|| {
                                    view! {
                                        <li
                                            class="tree-folder"
                                            style="padding-left: 0.75rem"
                                            on:click=move |_| {
                                                state
                                                    .collapsed_project_groups
                                                    .update(|set| {
                                                        if !set.remove("professional") {
                                                            set.insert("professional".to_string());
                                                        }
                                                    });
                                            }
                                        >
                                            <span class="folder-icon">
                                                {move || {
                                                    if state
                                                        .collapsed_project_groups
                                                        .get()
                                                        .contains("professional")
                                                    {
                                                        "\u{25b8}"
                                                    } else {
                                                        "\u{25be}"
                                                    }
                                                }}
                                            </span>
                                            " professional/"
                                        </li>
                                        <div style:display=move || {
                                            if state
                                                .collapsed_project_groups
                                                .get()
                                                .contains("professional")
                                            {
                                                "none"
                                            } else {
                                                "block"
                                            }
                                        }>{professional_views}</div>
                                    }
                                })}
                        </>
                    }
                }
            </ul>

            // ============================================================
            // Publications
            // ============================================================

            <div
                class=move || {
                    let path = location.pathname.get();
                    if path.starts_with("/publications") {
                        "sidebar-item sidebar-section active"
                    } else {
                        "sidebar-item sidebar-section"
                    }
                }
                on:click=move |_| {
                    state.is_publications_open.update(|open| *open = !*open);
                }
            >
                <span class="folder-icon">
                    {move || if state.is_publications_open.get() { "\u{25be}" } else { "\u{25b8}" }}
                </span>
                " "
                <A
                    href="/publications"
                    on:click=move |event| {
                        event.stop_propagation();
                        close_mobile();
                    }
                >
                    "publications"
                </A>
            </div>

            <ul
                class="file-tree"
                style:display=move || {
                    if state.is_publications_open.get() { "block" } else { "none" }
                }
            >
                {PUBLICATIONS
                    .iter()
                    .map(|entry| {
                        let title_plain = segments_plain_value(entry.title);
                        let href = format!("/publications#{}", title_plain);
                        let label = if segments_has_scrambled(entry.title) {
                            view! { <span class="teaser">{render_segments(entry.title)}</span> }
                                .into_any()
                        } else {
                            view! { {title_plain} }.into_any()
                        };
                        view! {
                            <li class="tree-post" style="padding-left: 0.75rem">
                                <A href=href on:click=move |_| close_mobile()>
                                    {label}
                                </A>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>

        </nav>
    };
}

// ============================================================
// Tree building helpers
// ============================================================

fn build_post_tree() -> TreeNode {
    let mut root = TreeNode::new();
    for post in POSTS {
        if post.folder.is_empty() {
            root.posts.push(post);
        } else {
            let mut node = &mut root;
            for segment in post.folder.split('/') {
                node = node.children.entry(segment.to_string()).or_insert_with(TreeNode::new);
            }
            node.posts.push(post);
        }
    }
    return root;
}

fn flatten_tree(node: &TreeNode, depth: usize, current_path: &str) -> Vec<TreeItem> {
    let mut items = Vec::new();
    for (name, child) in &node.children {
        let path = if current_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", current_path, name)
        };
        items.push(TreeItem::Folder {
            name: name.clone(),
            depth,
            path: path.clone(),
        });
        items.extend(flatten_tree(child, depth + 1, &path));
    }
    for post in &node.posts {
        items.push(TreeItem::PostEntry {
            post,
            depth,
            parent: current_path.to_string(),
        });
    }
    return items;
}

fn parent_folder_of(path: &str) -> String {
    match path.rfind('/') {
        Some(separator_position) => path[..separator_position].to_string(),
        None => String::new(),
    }
}

fn is_path_collapsed(path: &str, collapsed: &std::collections::BTreeSet<String>) -> bool {
    if path.is_empty() {
        return false;
    }
    let mut prefix = String::new();
    for segment in path.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(segment);
        if collapsed.contains(&prefix) {
            return true;
        }
    }
    return false;
}
