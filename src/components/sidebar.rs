#![allow(clippy::needless_return)]

use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};

use super::SidebarState;
use crate::models::post::{POSTS, Post};

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

fn build_post_tree() -> TreeNode {
    let mut root = TreeNode::new();
    for post in POSTS {
        if post.folder.is_empty() {
            root.posts.push(post);
        } else {
            let mut node = &mut root;
            for part in post.folder.split('/') {
                node = node.children.entry(part.to_string()).or_insert_with(TreeNode::new);
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

fn parent_of(path: &str) -> String {
    match path.rfind('/') {
        Some(pos) => path[..pos].to_string(),
        None => String::new(),
    }
}

fn is_path_collapsed(path: &str, collapsed: &std::collections::BTreeSet<String>) -> bool {
    if path.is_empty() {
        return false;
    }
    let mut prefix = String::new();
    for part in path.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(part);
        if collapsed.contains(&prefix) {
            return true;
        }
    }
    return false;
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<SidebarState>().expect("SidebarState context");
    let location = use_location();

    let tree = build_post_tree();
    let items = flatten_tree(&tree, 1, "");

    let close_mobile = move || {
        state.mobile_open.set(false);
    };

    return view! {
        <nav class="sidebar" class:open=move || state.mobile_open.get()>
            // About link
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

            // Blog section
            <div
                class="sidebar-item sidebar-section"
                on:click=move |_| {
                    state.blog_open.update(|open| *open = !*open);
                }
            >
                <span class="folder-icon">
                    {move || if state.blog_open.get() { "\u{25be}" } else { "\u{25b8}" }}
                </span>
                " blog"
            </div>

            <ul
                class="file-tree"
                style:display=move || if state.blog_open.get() { "block" } else { "none" }
            >
                {items
                    .into_iter()
                    .map(|item| match item {
                        TreeItem::Folder { name, depth, path } => {
                            let path_click = path.clone();
                            let path_icon = path.clone();
                            let parent = parent_of(&path);

                            view! {
                                <li
                                    class="tree-folder"
                                    style=move || {
                                        let pad = format!(
                                            "padding-left: {}rem",
                                            depth as f32 * 0.75,
                                        );
                                        if is_path_collapsed(&parent, &state.collapsed.get()) {
                                            format!("{pad}; display: none")
                                        } else {
                                            pad
                                        }
                                    }
                                    on:click=move |_| {
                                        state
                                            .collapsed
                                            .update(|s| {
                                                if !s.remove(&path_click) {
                                                    s.insert(path_click.clone());
                                                }
                                            });
                                    }
                                >
                                    <span class="folder-icon">
                                        {move || {
                                            if state.collapsed.get().contains(&path_icon) {
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
                                        let pad = format!(
                                            "padding-left: {}rem",
                                            depth as f32 * 0.75,
                                        );
                                        if is_path_collapsed(&parent, &state.collapsed.get()) {
                                            format!("{pad}; display: none")
                                        } else {
                                            pad
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

            // Projects link
            <A
                href="/projects"
                attr:class=move || {
                    let path = location.pathname.get();
                    if path == "/projects" { "sidebar-item active" } else { "sidebar-item" }
                }
                on:click=move |_| close_mobile()
            >
                "projects"
            </A>

            // Publications link
            <A
                href="/publications"
                attr:class=move || {
                    let path = location.pathname.get();
                    if path == "/publications" { "sidebar-item active" } else { "sidebar-item" }
                }
                on:click=move |_| close_mobile()
            >
                "publications"
            </A>

            // Legal links
            <A
                href="/imprint"
                attr:class=move || {
                    let path = location.pathname.get();
                    if path == "/imprint" { "sidebar-item active" } else { "sidebar-item" }
                }
                on:click=move |_| close_mobile()
            >
                "imprint"
            </A>
            <A
                href="/privacy"
                attr:class=move || {
                    let path = location.pathname.get();
                    if path == "/privacy" { "sidebar-item active" } else { "sidebar-item" }
                }
                on:click=move |_| close_mobile()
            >
                "privacy"
            </A>
        </nav>
    };
}
