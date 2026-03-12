#![allow(clippy::needless_return)]

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};

pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_MATH);

    let parser = Parser::new_ext(markdown, options);

    let mut special: Option<SpecialBlock> = None;
    let mut code_buf = String::new();
    let mut events: Vec<Event> = Vec::new();
    let mut in_code_block = false;

    for event in parser {
        if special.is_some() {
            match event {
                Event::Text(text) => code_buf.push_str(&text),
                Event::End(TagEnd::CodeBlock) => {
                    let escaped = html_escape(&code_buf);
                    let html_str = match special.take().unwrap() {
                        SpecialBlock::Tikz => format!("<pre class=\"tikz-src\">{escaped}</pre>"),
                        SpecialBlock::TikzCd => format!("<pre class=\"tikz-src\" data-libs=\"cd\">{escaped}</pre>"),
                        SpecialBlock::Mermaid => format!("<pre class=\"mermaid\">{escaped}</pre>"),
                    };
                    events.push(Event::Html(html_str.into()));
                }
                _ => {}
            }
            continue;
        }

        if in_code_block {
            match event {
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    events.push(Event::Html("</code></pre>".into()));
                }
                Event::Text(text) => {
                    events.push(Event::Html(html_escape(&text).into()));
                }
                _ => {}
            }
            continue;
        }

        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => match lang.as_ref() {
                "tikz" => {
                    special = Some(SpecialBlock::Tikz);
                    code_buf.clear();
                }
                "tikzcd" => {
                    special = Some(SpecialBlock::TikzCd);
                    code_buf.clear();
                }
                "mermaid" => {
                    special = Some(SpecialBlock::Mermaid);
                    code_buf.clear();
                }
                _ => {
                    in_code_block = true;
                    events.push(Event::Html(format!("<pre><code class=\"language-{}\">", lang).into()))
                }
            },
            Event::InlineMath(math) => {
                let escaped = html_escape(&math);
                events.push(Event::Html(
                    format!("<span class=\"math-inline\">\\({escaped}\\)</span>").into(),
                ));
            }
            Event::DisplayMath(math) => {
                let escaped = html_escape(&math);
                events.push(Event::Html(
                    format!("<div class=\"math-display\">\\[{escaped}\\]</div>").into(),
                ));
            }
            other => events.push(other),
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());
    return html_output;
}

enum SpecialBlock {
    Tikz,
    TikzCd,
    Mermaid,
}

fn html_escape(s: &str) -> String {
    return s
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
}
