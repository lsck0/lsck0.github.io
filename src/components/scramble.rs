use std::time::Duration;

use leptos::prelude::*;

const SCRAMBLE_CHARACTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789!@#%&*~^+-=<>/|{}[]()_:;?.,";
const TICK_INTERVAL_MS: u64 = 75;

fn random_character() -> char {
    let index = (js_sys::Math::random() * SCRAMBLE_CHARACTERS.len() as f64) as usize;
    return SCRAMBLE_CHARACTERS[index % SCRAMBLE_CHARACTERS.len()] as char;
}

#[component]
pub fn ScrambleText(len: usize) -> impl IntoView {
    let (displayed_text, set_displayed_text) = signal(String::new());

    Effect::new(move |_: Option<()>| {
        let initial_text: String = (0..len).map(|_| random_character()).collect();
        set_displayed_text.set(initial_text);

        let interval_handle = set_interval_with_handle(
            move || {
                let scrambled_text: String = (0..len).map(|_| random_character()).collect();
                set_displayed_text.set(scrambled_text);
            },
            Duration::from_millis(TICK_INTERVAL_MS),
        );

        on_cleanup(move || {
            if let Ok(handle) = interval_handle {
                handle.clear();
            }
        });
    });

    return view! { <span class="scramble-text">{displayed_text}</span> };
}
