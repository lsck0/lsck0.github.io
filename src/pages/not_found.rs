#![allow(clippy::needless_return)]

use std::time::Duration;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::A;

use crate::components::{footer::Footer, header::Header};

#[component]
pub fn NotFound() -> impl IntoView {
    let (countdown, set_countdown) = signal(5u32);

    Effect::new(move |_: Option<()>| {
        fn tick(countdown: ReadSignal<u32>, set_countdown: WriteSignal<u32>) {
            set_timeout(
                move || {
                    let remaining = countdown.get_untracked();
                    if remaining <= 1 {
                        if let Some(window) = web_sys::window() {
                            let _ = window.location().set_href("/");
                        }
                    } else {
                        set_countdown.set(remaining - 1);
                        tick(countdown, set_countdown);
                    }
                },
                Duration::from_secs(1),
            );
        }
        tick(countdown, set_countdown);
    });

    return view! {
        <Title text="\u{03bb} lsck0 \u{2014} 404" />
        <Header />
        <main class="not-found">
            <h1>"404"</h1>
            <p>"This page does not exist."</p>
            <A href="/">"go home"</A>
            <p class="redirect-notice">"(redirecting in " {countdown} " seconds)"</p>
        </main>
        <Footer />
    };
}
