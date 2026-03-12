#![allow(clippy::needless_return)]

use leptos::prelude::*;
use leptos_meta::*;

use crate::components::layout::Layout;

#[component]
pub fn Privacy() -> impl IntoView {
    return view! {
        <Title text="\u{03bb} lsck0 \u{2014} privacy" />
        <Layout>
            <div class="legal-page">
                <h1>"Privacy Policy"</h1>

                <section>
                    <h2>"1. Overview"</h2>
                    <p>
                        "This website is served as a static site via GitHub Pages. "
                        "No personal data is collected, stored, or processed by the operator."
                    </p>
                </section>

                <section>
                    <h2>"2. Hosting"</h2>
                    <p>
                        "This website is hosted on GitHub Pages. The hosting provider may "
                        "collect server log files that document your visit. "
                        "Details can be found in the "
                        <a
                            href="https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement"
                            target="_blank"
                        >
                            "GitHub Privacy Statement"
                        </a> "."
                    </p>
                </section>

                <section>
                    <h2>"3. External Resources"</h2>
                    <p>
                        "This website loads fonts and JavaScript libraries from CDN services "
                        "(jsDelivr, tikzjax.com). When fetching these resources, your IP address "
                        "is transmitted to the respective providers."
                    </p>
                </section>

                <section>
                    <h2>"4. Local Storage"</h2>
                    <p>
                        "This website stores your color scheme preference (light/dark) in "
                        "your browser's localStorage. This data does not leave your browser."
                    </p>
                </section>

                <section>
                    <h2>"5. Your Rights"</h2>
                    <p class="placeholder">"TODO: Add contact details for privacy inquiries."</p>
                </section>
            </div>
        </Layout>
    };
}
