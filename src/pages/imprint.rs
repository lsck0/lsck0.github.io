#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_meta::*;

use crate::components::layout::Layout;

#[component]
pub fn Imprint() -> impl IntoView {
    return view! {
        <Title text="\u{03bb} lsck0 \u{2014} imprint" />
        <Layout>
            <div class="legal-page">
                <h1>"Imprint"</h1>

                <section>
                    <h2>"Information pursuant to \u{00a7} 5 TMG"</h2>
                    <p class="placeholder">"TODO: Add name and address."</p>
                </section>

                <section>
                    <h2>"Contact"</h2>
                    <p class="placeholder">"TODO: Add email address."</p>
                </section>

                <section>
                    <h2>"Disclaimer"</h2>
                    <h3>"Liability for Content"</h3>
                    <p>
                        "The contents of these pages were created with the utmost care. "
                        "However, no guarantee can be given for the accuracy, completeness, "
                        "and timeliness of the content."
                    </p>
                    <h3>"Liability for Links"</h3>
                    <p>
                        "This website contains links to external third-party websites over whose "
                        "content there is no influence. The respective provider or operator of the "
                        "linked pages is always responsible for their content."
                    </p>
                </section>
            </div>
        </Layout>
    };
}
