use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::A;

use crate::{components::layout::Layout, models::meta::META};

#[component]
pub fn NotFoundPage() -> impl IntoView {
    return view! {
        <Title text=format!("\u{03bb} {} \u{2014} 404", META.title) />
        <Layout>
            <main class="not-found">
                <div class="not-found-glitch" data-text="404">
                    "404"
                </div>
                <p class="not-found-message">
                    "the page you're looking for has vanished into the void"
                </p>
                <div class="not-found-suggestions">
                    <p class="not-found-hint">"perhaps you meant to visit:"</p>
                    <nav class="not-found-links">
                        <A href="/">"home"</A>
                        <A href="/blog">"blog"</A>
                        <A href="/about">"about"</A>
                    </nav>
                </div>
            </main>
        </Layout>
    };
}
