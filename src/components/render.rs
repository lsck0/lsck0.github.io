use std::collections::HashMap;

use leptos::prelude::*;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};

use crate::models::post::POSTS;

// ============================================================
// Public API
// ============================================================

/// Calls the JS `renderPost()` function after the current component mounts.
/// Used by post pages and prose pages to initialize tooltips, math, diagrams, etc.
pub fn call_render_post() {
    Effect::new(move |_: Option<()>| {
        let _ = js_sys::eval("renderPost();");
    });
}

pub struct RenderedMarkdown {
    pub html: String,
    pub link_occurrences: HashMap<String, Vec<String>>,
}

pub fn markdown_to_html(markdown: &str) -> RenderedMarkdown {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_MATH | Options::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(markdown, options);
    let (events, mut link_occurrences) = process_events(parser);

    let mut output = String::new();
    html::push_html(&mut output, events.into_iter());
    output = process_callouts(&output);
    output = process_labeled_blocks(&output);

    // Post-process to track xref links that were rendered as HTML
    output = extract_xref_links(&output, &mut link_occurrences);

    return RenderedMarkdown {
        html: output,
        link_occurrences,
    };
}

// ============================================================
// Special fenced code blocks
// ============================================================

enum SpecialBlock {
    Tikz,
    TikzCd,
    Mermaid,
    LabeledBlock,
}

fn special_block(language: &str) -> Option<SpecialBlock> {
    match language {
        "tikz" => Some(SpecialBlock::Tikz),
        "tikzcd" => Some(SpecialBlock::TikzCd),
        "mermaid" => Some(SpecialBlock::Mermaid),
        "definition" | "theorem" | "lemma" | "corollary" | "proposition" | "axiom" | "remark" | "example"
        | "conjecture" | "exercise" | "problem" | "proof" => Some(SpecialBlock::LabeledBlock),
        _ => None,
    }
}

impl SpecialBlock {
    fn render(&self, content: &str) -> String {
        let escaped = html_escape(content);
        match self {
            SpecialBlock::Tikz => format!("<pre class=\"tikz-src\">{escaped}</pre>"),
            SpecialBlock::TikzCd => {
                format!("<pre class=\"tikz-src\" data-libs=\"cd\">{escaped}</pre>")
            }
            SpecialBlock::Mermaid => format!("<pre class=\"mermaid\">{content}</pre>"),
            SpecialBlock::LabeledBlock => content.to_string(),
        }
    }
}

// ============================================================
// Event processing state machine
// ============================================================

enum State {
    Normal,
    Code(String),
    Special(SpecialBlock, String),
    Image(String, String, u32),
    MediaLink(String, String, String),
    /// Heading(level, plain_text_for_slug, html_content)
    Heading(u8, String, String),
}

fn detect_media_type(url: &str) -> Option<String> {
    let lowercase_url = url.to_lowercase();
    if lowercase_url.ends_with(".mp3")
        || lowercase_url.ends_with(".wav")
        || lowercase_url.ends_with(".ogg")
        || lowercase_url.ends_with(".flac")
    {
        return Some("audio".to_string());
    }
    if lowercase_url.ends_with(".mp4") || lowercase_url.ends_with(".webm") || lowercase_url.ends_with(".ogv") {
        return Some("video".to_string());
    }
    if lowercase_url.ends_with(".pdf") {
        return Some("pdf".to_string());
    }
    return None;
}

fn process_events<'a>(parser: impl Iterator<Item = Event<'a>>) -> (Vec<Event<'a>>, HashMap<String, Vec<String>>) {
    let mut events: Vec<Event<'a>> = Vec::new();
    let mut state = State::Normal;
    let mut link_counter: u32 = 0;
    let mut image_counter: u32 = 0;
    let mut link_occurrences: HashMap<String, Vec<String>> = HashMap::new();

    for event in parser {
        state = match state {
            State::Special(block, mut buffer) => match event {
                Event::Text(text) => {
                    buffer.push_str(&text);
                    State::Special(block, buffer)
                }
                Event::End(TagEnd::CodeBlock) => {
                    events.push(Event::Html(block.render(&buffer).into()));
                    State::Normal
                }
                _ => State::Special(block, buffer),
            },

            State::Heading(level, mut plain, mut html_buf) => match event {
                Event::Text(text) => {
                    plain.push_str(&text);
                    html_buf.push_str(&html_escape(&text));
                    State::Heading(level, plain, html_buf)
                }
                Event::InlineMath(math) => {
                    plain.push_str(&math);
                    let escaped = html_escape(&math);
                    html_buf.push_str(&format!(
                        "<span class=\"math-inline\" data-latex=\"{escaped}\">\\({escaped}\\)</span>"
                    ));
                    State::Heading(level, plain, html_buf)
                }
                Event::Code(code) => {
                    plain.push_str(&code);
                    html_buf.push_str(&format!("<code>{}</code>", html_escape(&code)));
                    State::Heading(level, plain, html_buf)
                }
                Event::End(TagEnd::Heading(_)) => {
                    let id = slugify(&plain);
                    events.push(Event::Html(
                        format!("<h{level} id=\"{id}\">{html_buf}</h{level}>").into(),
                    ));
                    State::Normal
                }
                _ => State::Heading(level, plain, html_buf),
            },

            State::Image(url, mut alt_text, id) => match event {
                Event::Text(text) => {
                    alt_text.push_str(&text);
                    State::Image(url, alt_text, id)
                }
                Event::End(TagEnd::Image) => {
                    let escaped_alt = html_escape(&alt_text);
                    events.push(Event::Html(
                        format!(
                            "<figure id=\"img-{id}\" class=\"post-figure\"><img src=\"{url}\" alt=\"{escaped_alt}\" \
                             loading=\"lazy\" /></figure>"
                        )
                        .into(),
                    ));
                    State::Normal
                }
                _ => State::Image(url, alt_text, id),
            },

            State::MediaLink(url, media_type, mut title) => match event {
                Event::Text(text) => {
                    title.push_str(&text);
                    State::MediaLink(url, media_type, title)
                }
                Event::End(TagEnd::Link) => {
                    let escaped_title = html_escape(&title);
                    events.push(Event::Html(
                        format!(
                            "<div class=\"media-embed\" data-type=\"{media_type}\" data-src=\"{url}\" \
                             data-title=\"{escaped_title}\"></div>"
                        )
                        .into(),
                    ));
                    State::Normal
                }
                _ => State::MediaLink(url, media_type, title),
            },

            State::Code(ref language) => match event {
                Event::End(TagEnd::CodeBlock) => {
                    events.push(Event::Html("</code></pre>".into()));
                    State::Normal
                }
                Event::Text(text) => {
                    let escaped = if language == "diff" {
                        render_diff_code(&text)
                    } else {
                        html_escape(&text)
                    };
                    events.push(Event::Html(escaped.into()));
                    State::Code(language.to_string())
                }
                _ => State::Code(language.to_string()),
            },

            State::Normal => match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    let numeric_level = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    State::Heading(numeric_level, String::new(), String::new())
                }
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language))) => {
                    let language_string = language.to_string();
                    if let Some(block) = special_block(&language_string) {
                        State::Special(block, String::new())
                    } else {
                        let css_class = if language_string == "diff" {
                            "language-diff".to_string()
                        } else {
                            format!("language-{language_string}")
                        };
                        events.push(Event::Html(format!("<pre><code class=\"{css_class}\">").into()));
                        State::Code(language_string)
                    }
                }
                Event::InlineMath(math) => {
                    let escaped = html_escape(&math);
                    events.push(Event::Html(
                        format!("<span class=\"math-inline\" data-latex=\"{escaped}\">\\({escaped}\\)</span>").into(),
                    ));
                    State::Normal
                }
                Event::DisplayMath(math) => {
                    let escaped = html_escape(&math);
                    events.push(Event::Html(
                        format!("<div class=\"math-display\" data-latex=\"{escaped}\">\\[{escaped}\\]</div>").into(),
                    ));
                    State::Normal
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    let url = dest_url.to_string();

                    if let Some(media_type) = detect_media_type(&url) {
                        State::MediaLink(url, media_type, String::new())
                    } else {
                        let is_trackable = url.starts_with("/blog/") || url.starts_with("http");
                        if is_trackable {
                            link_counter += 1;
                            let anchor_id = format!("ref-{link_counter}");
                            link_occurrences.entry(url.clone()).or_default().push(anchor_id.clone());

                            let metadata_attributes = if let Some(slug) = url.strip_prefix("/blog/") {
                                let slug = slug.split('#').next().unwrap_or(slug);
                                POSTS
                                    .iter()
                                    .find(|post| post.slug == slug)
                                    .map(|post| {
                                        let description = html_escape(post.description());
                                        let tags = post.tags().join(", ");
                                        let series = post.series().unwrap_or("");
                                        format!(
                                            " data-post-title=\"{}\" data-post-desc=\"{description}\" \
                                             data-post-tags=\"{tags}\" data-post-series=\"{series}\"",
                                            html_escape(post.title())
                                        )
                                    })
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            let url_without_fragment = url.split('#').next().unwrap_or(&url);
                            link_occurrences
                                .entry(url_without_fragment.to_string())
                                .or_default()
                                .push(anchor_id.clone());
                            events.push(Event::Html(
                                format!(
                                    "<a href=\"{url}\" id=\"{anchor_id}\"{metadata_attributes} target=\"_blank\" \
                                     rel=\"noopener\">"
                                )
                                .into(),
                            ));
                        } else {
                            events.push(Event::Html(
                                format!("<a href=\"{url}\" target=\"_blank\" rel=\"noopener\">").into(),
                            ));
                        }
                        State::Normal
                    }
                }
                Event::End(TagEnd::Link) => {
                    events.push(Event::Html("</a>".into()));
                    State::Normal
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    image_counter += 1;
                    let url = dest_url.to_string();

                    if url.starts_with("/blog/") || url.starts_with("http") {
                        link_counter += 1;
                        let anchor_id = format!("ref-{link_counter}");
                        link_occurrences.entry(url.clone()).or_default().push(anchor_id);
                    }

                    State::Image(url, String::new(), image_counter)
                }
                other => {
                    events.push(other);
                    State::Normal
                }
            },
        };
    }

    return (events, link_occurrences);
}

// ============================================================
// Diff code highlighting
// ============================================================

fn render_diff_code(text: &str) -> String {
    let mut result = String::new();
    for line in text.lines() {
        let escaped = html_escape(line);
        if line.starts_with('+') {
            result.push_str(&format!("<span class=\"diff-add\">{escaped}</span>\n"));
        } else if line.starts_with('-') {
            result.push_str(&format!("<span class=\"diff-del\">{escaped}</span>\n"));
        } else {
            result.push_str(&escaped);
            result.push('\n');
        }
    }
    return result;
}

// ============================================================
// Callout post-processing
// ============================================================

/// Converts blockquotes starting with `[!TYPE]` into styled admonition blocks.
fn process_callouts(html: &str) -> String {
    let callout_types = ["info", "warning", "tip", "danger", "note"];
    let mut result = html.to_string();

    for callout_type in &callout_types {
        let marker = format!("[!{callout_type}]");
        let uppercase_label = callout_type.to_uppercase();

        let search_with_newline = format!("<blockquote>\n<p>{marker}");
        let replacement = format!(
            "<div class=\"callout callout-{callout_type}\"><div class=\"callout-title\">{uppercase_label}</div><div \
             class=\"callout-body\"><p>"
        );
        result = result.replace(&search_with_newline, &replacement);

        let search_with_double_newline = format!("<blockquote>\n<p>{marker}\n");
        let replacement_alt = format!(
            "<div class=\"callout callout-{callout_type}\"><div class=\"callout-title\">{uppercase_label}</div><div \
             class=\"callout-body\"><p>"
        );
        result = result.replace(&search_with_double_newline, &replacement_alt);
    }

    let mut processed = String::new();
    let mut inside_callout = false;
    for line in result.lines() {
        if line.contains("class=\"callout ") {
            inside_callout = true;
        }
        if inside_callout && line.contains("</blockquote>") {
            processed.push_str(&line.replace("</blockquote>", "</div></div>"));
            processed.push('\n');
            inside_callout = false;
        } else {
            processed.push_str(line);
            processed.push('\n');
        }
    }

    return processed;
}

// ============================================================
// Labeled block post-processing
// ============================================================

/// Converts `<!--BLOCK|kind|id|number|title-->...<!--/BLOCK-->` markers into styled divs.
fn process_labeled_blocks(html: &str) -> String {
    let mut result = String::new();
    let mut remaining = html;

    while let Some(start_position) = remaining.find("<!--BLOCK|") {
        result.push_str(&remaining[..start_position]);
        let after_marker = &remaining[start_position..];

        if let Some(header_end) = after_marker.find("-->") {
            let header = &after_marker[10..header_end];
            let after_header = &after_marker[header_end + 3..];

            if let Some(block_end) = after_header.find("<!--/BLOCK-->") {
                let content = after_header[..block_end].trim();
                let parts: Vec<&str> = header.splitn(4, '|').collect();

                if parts.len() == 4 {
                    let kind = parts[0];
                    let id = parts[1];
                    let number = parts[2];
                    let title = parts[3].replace("\\|", "|");

                    let kind_display = capitalize_first(kind);
                    let is_proof = kind == "proof";

                    let header_html = if is_proof {
                        if title.is_empty() {
                            "<strong>Proof.</strong>".to_string()
                        } else {
                            format!("<strong>Proof</strong> ({title})<strong>.</strong>")
                        }
                    } else if !number.is_empty() {
                        if title.is_empty() {
                            format!("<strong>{kind_display} {number}.</strong>")
                        } else {
                            format!("<strong>{kind_display} {number}</strong> ({title})<strong>.</strong>")
                        }
                    } else {
                        format!("<strong>{kind_display}.</strong>")
                    };

                    let qed_marker = if is_proof {
                        "\n<span class=\"qed\">\u{220e}</span>"
                    } else {
                        ""
                    };

                    result.push_str(&format!(
                        "<div class=\"labeled-block {kind}\" id=\"{id}\"><div \
                         class=\"labeled-block-header\">{header_html}</div><div \
                         class=\"labeled-block-content\">{content}</div>{qed_marker}</div>"
                    ));
                } else {
                    result.push_str(content);
                }

                remaining = &after_header[block_end + 13..];
            } else {
                result.push_str(&remaining[..start_position + header_end + 3]);
                remaining = after_header;
            }
        } else {
            result.push_str(&remaining[..start_position + 10]);
            remaining = &after_marker[10..];
        }
    }

    result.push_str(remaining);
    return result;
}

// ============================================================
// Text transformation helpers
// ============================================================

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else if character == ' ' || character == '-' || character == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&character| character != '\0')
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn capitalize_first(text: &str) -> String {
    let mut characters = text.chars();
    match characters.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
    }
}

fn html_escape(input: &str) -> String {
    return input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
}

// ============================================================
// XRef link tracking
// ============================================================

fn extract_xref_links(html: &str, link_occurrences: &mut HashMap<String, Vec<String>>) -> String {
    let mut result = html.to_string();
    let mut link_counter = 1000;

    for class in ["xref", "auto-def"] {
        let search_pattern = format!("class=\"{}\"", class);
        let mut offsets_to_insert: Vec<(usize, String)> = Vec::new();

        let mut search_start = 0;
        while let Some(class_pos) = result[search_start..].find(&search_pattern) {
            let absolute_pos = search_start + class_pos;
            let after_class = absolute_pos + search_pattern.len();

            let href_start = match result[after_class..].find("href=\"") {
                Some(pos) => after_class + pos + 6,
                None => {
                    search_start = absolute_pos + 1;
                    continue;
                }
            };
            let href_end = match result[href_start..].find('"') {
                Some(pos) => href_start + pos,
                None => {
                    search_start = absolute_pos + 1;
                    continue;
                }
            };

            let href = &result[href_start..href_end].to_string();
            if href.starts_with("/blog/") || href.starts_with("http") {
                link_counter += 1;
                let anchor_id = format!("ref-{}", link_counter);
                let url_without_fragment = href.split('#').next().unwrap_or(href);
                link_occurrences
                    .entry(url_without_fragment.to_string())
                    .or_default()
                    .push(anchor_id.clone());
                offsets_to_insert.push((absolute_pos, format!(" id=\"{}\"", anchor_id)));
            }

            search_start = absolute_pos + 1;
        }

        for (pos, insert_text) in offsets_to_insert.into_iter().rev() {
            result.insert_str(pos, &insert_text);
        }
    }

    result
}
