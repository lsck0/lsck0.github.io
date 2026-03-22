use leptos::prelude::*;
use leptos_router::components::A;

use super::toggle_theme;

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_DATE: &str = env!("GIT_DATE");

#[component]
pub fn Footer() -> impl IntoView {
    let on_toggle_theme = move |_| {
        toggle_theme();
    };

    return view! {
        <footer class="site-footer">
            <div class="footer-meta">
                {GIT_DATE} <span class="footer-sep">{"\u{00b7}"}</span>
                <span class="commit-info">
                    "commit "
                    <a
                        href=format!("https://github.com/lsck0/lsck0.github.io/commit/{GIT_HASH}")
                        target="_blank"
                    >
                        {GIT_HASH}
                    </a>
                </span> <span class="footer-sep">{"\u{00b7}"}</span>
                <a href="https://github.com/lsck0/lsck0.github.io" target="_blank">
                    "source"
                </a> <span class="footer-sep">{"\u{00b7}"}</span>
                <A href="/rss.xml" attr:title="RSS Feed">
                    "rss"
                </A> <span class="footer-sep">{"\u{00b7}"}</span>
                <button class="theme-toggle-footer" on:click=on_toggle_theme title="Toggle theme">
                    "toggle theme"
                </button>
            </div>
            <div class="footer-legal">
                <A href="/imprint">"imprint"</A>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <A href="/privacy">"privacy"</A>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <A href="/tos">"terms"</A>
            </div>
        </footer>
    };
}
