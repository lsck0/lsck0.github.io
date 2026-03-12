#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_router::components::A;

use super::toggle_theme;

const GIT_HASH: Option<&str> = option_env!("GIT_HASH");
const GIT_DATE: Option<&str> = option_env!("GIT_DATE");

#[component]
pub fn Footer() -> impl IntoView {
    let hash = GIT_HASH.unwrap_or("dev");
    let date = GIT_DATE.unwrap_or("");

    let on_toggle_theme = move |_| {
        toggle_theme();
    };

    return view! {
        <footer class="site-footer">
            <div class="footer-meta">
                {if !date.is_empty() { date.to_string() } else { String::new() }}
                <span class="footer-sep">{"\u{00b7}"}</span>
                <span class="commit-info">
                    "commit "
                    <a
                        href=format!("https://github.com/lsck0/lsck0.github.io/commit/{hash}")
                        target="_blank"
                    >
                        {hash}
                    </a>
                </span> <span class="footer-sep">{"\u{00b7}"}</span>
                <a href="https://github.com/lsck0/lsck0.github.io" target="_blank">
                    "source code"
                </a> <span class="footer-sep">{"\u{00b7}"}</span>
                <a class="theme-toggle-footer" on:click=on_toggle_theme title="Toggle theme">
                    "toggle theme"
                </a>
            </div>
            <div class="footer-legal">
                <A href="/imprint">"imprint"</A>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <A href="/privacy">"privacy"</A>
            </div>
        </footer>
    };
}
