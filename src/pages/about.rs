#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_meta::*;

use crate::components::layout::Layout;

#[component]
pub fn About() -> impl IntoView {
    return view! {
        <Title text="\u{03bb} lsck0 \u{2014} about" />
        <Layout>
            <div class="about">
                <h1>"About"</h1>

                <section>
                    <p>"Welcome. This is my personal corner of the internet."</p>
                    <p class="placeholder">"TODO: Write a short introduction about yourself."</p>
                </section>

                <section>
                    <h2>"Background"</h2>
                    <p class="placeholder">
                        "TODO: Share your background, education, and professional experience."
                    </p>
                </section>

                <section>
                    <h2>"Interests"</h2>
                    <p class="placeholder">
                        "TODO: What are you passionate about? List your interests here."
                    </p>
                </section>

                <section>
                    <h2>"Contact"</h2>
                    <p class="placeholder">"TODO: Add your contact information or social links."</p>
                </section>
            </div>
        </Layout>
    };
}
