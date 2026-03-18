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
                <pre class="ascii-404">
                    {r#"
                    ██╗  ██╗ ██████╗ ██╗  ██╗
                    ██║  ██║██╔═████╗██║  ██║
                    ███████║██║██╔██║███████║
                    ╚════██║████╔╝██║╚════██║
                    ██║╚██████╔╝     ██║
                    ╚═╝ ╚═════╝      ╚═╝
                    "#}
                </pre>
                <p>"This page does not exist."</p>
                <A href="/">"go home"</A>
            </main>
        </Layout>
    };
}
