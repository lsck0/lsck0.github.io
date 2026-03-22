use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::A;

use crate::{components::layout::Layout, models::meta::META};

const ASCII_404: &str = r#" _  _    ___  _  _
| || |  / _ \| || |
| || |_| | | | || |_
|__   _| | | |__   _|
   | | | |_| |  | |
   |_|  \___/   |_|"#;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    return view! {
        <Title text=format!("\u{03bb} {} \u{2014} 404", META.title) />
        <Layout>
            <main class="not-found">
                <pre class="ascii-404">{ASCII_404}</pre>
                <p>"The page you're looking for doesn't exist."</p>
                <A href="/">"go home"</A>
            </main>
        </Layout>
    };
}
