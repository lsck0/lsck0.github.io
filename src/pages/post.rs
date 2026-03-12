#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::A, hooks::use_params_map};

use crate::{
    components::{SidebarState, layout::Layout, render::markdown_to_html},
    models::post::POSTS,
};

#[component]
pub fn PostPage() -> impl IntoView {
    let params = use_params_map();

    let post = move || {
        params.with(|map| {
            let slug = map.get("slug")?;
            POSTS.iter().find(|p| p.slug == slug)
        })
    };

    Effect::new(move |_: Option<()>| {
        if let Some(p) = post() {
            // Expand the sidebar blog section and uncollapse the post's folder hierarchy
            if let Some(state) = use_context::<SidebarState>() {
                state.blog_open.set(true);
                if !p.folder.is_empty() {
                    state.collapsed.update(|collapsed| {
                        let mut prefix = String::new();
                        for (i, part) in p.folder.split('/').enumerate() {
                            if i > 0 {
                                prefix.push('/');
                            }
                            prefix.push_str(part);
                            collapsed.remove(&prefix);
                        }
                    });
                }
            }
            let _ = js_sys::eval("renderPost()");
        }
    });

    return view! {
        {move || match post() {
            None => {
                view! {
                    <Title text="\u{03bb} lsck0 \u{2014} not found" />
                    <Layout>
                        <div class="not-found">
                            <h1>"404"</h1>
                            <p>"This post does not exist."</p>
                            <A href="/blog">"back to blog"</A>
                        </div>
                    </Layout>
                }
                    .into_any()
            }
            Some(p) => {
                let html = markdown_to_html(p.content);

                view! {
                    <Title text=format!("\u{03bb} lsck0 \u{2014} {}", p.title()) />
                    <Layout>
                        <article id="post-content">
                            <h1 class="post-title">{p.title()}</h1>
                            <div class="post-meta">
                                <span class="date">{p.date_formatted()}</span>
                                <span class="post-tags">
                                    {p
                                        .tags()
                                        .into_iter()
                                        .map(|t| {
                                            let href = format!("/blog?tag={t}");
                                            view! {
                                                <A href=href attr:class="tag">
                                                    {format!("#{t}")}
                                                </A>
                                            }
                                        })
                                        .collect_view()}
                                </span>
                            </div>
                            <div class="content" inner_html=html />
                        </article>
                    </Layout>
                }
                    .into_any()
            }
        }}
    };
}
