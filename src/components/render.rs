#![allow(clippy::needless_return)]

//! IR → HTML rendering and JS post-processing trigger.
//!
//! Converts the IR block/inline tree into HTML strings with data attributes
//! for JS post-processing (KaTeX, TikZ, Mermaid, Prism, tooltips, pin buttons).

use std::collections::HashMap;

pub use ir::capitalize;
use ir::types::*;
use leptos::prelude::*;

use crate::models::post::{POSTS, Post};

// ============================================================
// Public API
// ============================================================

/// Calls the JS `renderPost()` function after the current component mounts.
pub fn call_render_post() {
    Effect::new(move |_: Option<()>| {
        let _ = js_sys::eval("renderPost();");
    });
}

/// Metadata about an internal post, used for hover preview data attributes.
pub struct PostPreview {
    pub title: String,
    pub description: String,
    pub tags: String,
    pub series: String,
}

/// Render an IR content tree to HTML, wired to the static POSTS data
/// for internal link hover previews.
pub fn render_post_content(content: &[Block], _post: &Post) -> (String, HashMap<String, Vec<String>>) {
    return render_content(content, |slug| {
        let target = POSTS.iter().find(|p| p.slug() == slug)?;
        Some(PostPreview {
            title: target.title().to_string(),
            description: target.description().to_string(),
            tags: target.tags().join(", "),
            series: target.series().unwrap_or("").to_string(),
        })
    });
}

/// Render an IR content tree to an HTML string.
///
/// `post_lookup` resolves internal `/blog/{slug}` links to preview metadata.
/// Returns `(html, link_occurrences)` where link_occurrences maps each URL
/// to the list of anchor IDs where it appears (for backlink tracking).
pub fn render_content(
    content: &[Block],
    post_lookup: impl Fn(&str) -> Option<PostPreview>,
) -> (String, HashMap<String, Vec<String>>) {
    let mut ctx = RenderContext::new(post_lookup);
    let html = render_blocks(content, &mut ctx);
    return (html, ctx.link_occurrences);
}

// ============================================================
// Render context
// ============================================================

struct RenderContext<F> {
    link_counter: u32,
    image_counter: u32,
    link_occurrences: HashMap<String, Vec<String>>,
    post_lookup: F,
    footnote_numbers: HashMap<String, u32>,
    footnote_counter: u32,
}

impl<F: Fn(&str) -> Option<PostPreview>> RenderContext<F> {
    fn new(post_lookup: F) -> Self {
        return Self {
            link_counter: 0,
            image_counter: 0,
            link_occurrences: HashMap::new(),
            post_lookup,
            footnote_numbers: HashMap::new(),
            footnote_counter: 0,
        };
    }

    fn footnote_number(&mut self, id: &str) -> u32 {
        if let Some(&num) = self.footnote_numbers.get(id) {
            return num;
        }
        self.footnote_counter += 1;
        self.footnote_numbers.insert(id.to_string(), self.footnote_counter);
        return self.footnote_counter;
    }

    fn next_link_id(&mut self, url: &str) -> String {
        self.link_counter += 1;
        let anchor_id = format!("ref-{}", self.link_counter);
        let tracking_url = url.split('#').next().unwrap_or(url).to_string();
        self.link_occurrences
            .entry(tracking_url)
            .or_default()
            .push(anchor_id.clone());
        return anchor_id;
    }

    fn next_image_id(&mut self) -> u32 {
        self.image_counter += 1;
        return self.image_counter;
    }

    fn post_metadata_attrs(&self, url: &str) -> String {
        let Some(slug) = url.strip_prefix("/blog/") else {
            return String::new();
        };
        let slug = slug.split('#').next().unwrap_or(slug);
        let Some(preview) = (self.post_lookup)(slug) else {
            return String::new();
        };
        return format!(
            " data-post-title=\"{}\" data-post-desc=\"{}\" data-post-tags=\"{}\" data-post-series=\"{}\"",
            html_escape(&preview.title),
            html_escape(&preview.description),
            preview.tags,
            preview.series,
        );
    }
}

// ============================================================
// Block rendering
// ============================================================

fn render_blocks<F: Fn(&str) -> Option<PostPreview>>(blocks: &[Block], ctx: &mut RenderContext<F>) -> String {
    let mut html = String::new();
    for block in blocks {
        html.push_str(&render_block(block, ctx));
    }
    return html;
}

fn render_block<F: Fn(&str) -> Option<PostPreview>>(block: &Block, ctx: &mut RenderContext<F>) -> String {
    match block {
        Block::Heading {
            level,
            id,
            number,
            children,
        } => {
            let inner = render_inlines(children, ctx);
            if number.is_empty() {
                format!("<h{level} id=\"{id}\">{inner}</h{level}>\n")
            } else {
                let num_span = format!("<span class=\"heading-num\">{number}</span>");
                format!("<h{level} id=\"{id}\">{num_span}{inner}</h{level}>\n")
            }
        }

        Block::Paragraph(children) => {
            let inner = render_inlines(children, ctx);
            format!("<p>{inner}</p>\n")
        }

        Block::CodeBlock { language, code } => {
            let escaped = html_escape(code);
            if language.is_empty() {
                format!("<pre><code>{escaped}</code></pre>\n")
            } else if language == "diff" {
                format!(
                    "<pre><code class=\"language-diff\">{}</code></pre>\n",
                    render_diff_code(code)
                )
            } else {
                format!("<pre><code class=\"language-{language}\">{escaped}</code></pre>\n")
            }
        }

        Block::LabeledBlock {
            kind,
            id,
            number,
            title,
            content,
            ..
        } => {
            let kind_display = capitalize(kind);
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

            let inner = render_blocks(content, ctx);
            let qed = if is_proof {
                "\n<span class=\"qed\">\u{220e}</span>"
            } else {
                ""
            };

            format!(
                "<div class=\"labeled-block {kind}\" id=\"{id}\"><div \
                 class=\"labeled-block-header\">{header_html}</div><div \
                 class=\"labeled-block-content\">{inner}</div>{qed}</div>\n"
            )
        }

        Block::Equation {
            id,
            number,
            title,
            latex,
            ..
        } => {
            let escaped = html_escape(latex);
            let tag = if !number.is_empty() {
                format!("\\tag{{{number}}}")
            } else {
                String::new()
            };
            let caption = if !title.is_empty() && !number.is_empty() {
                format!(
                    "<div class=\"labeled-block-header\"><strong>Equation {number}</strong> \
                     ({title})<strong>.</strong></div>"
                )
            } else {
                String::new()
            };
            format!(
                "<div class=\"labeled-block equation\" id=\"{id}\">{caption}<div class=\"math-display\" \
                 data-latex=\"{escaped}\">\\[{tag}{escaped}\\]</div></div>\n"
            )
        }

        Block::Diagram {
            variant,
            id,
            title,
            source,
            ..
        } => {
            let escaped = html_escape(source);
            let inner = match variant {
                DiagramKind::Tikz => format!("<pre class=\"tikz-src\">{escaped}</pre>"),
                DiagramKind::TikzCd => {
                    format!("<pre class=\"tikz-src\" data-libs=\"cd\">{escaped}</pre>")
                }
                DiagramKind::Mermaid => format!("<pre class=\"mermaid\">{source}</pre>"),
            };
            let caption = if !title.is_empty() {
                format!("<figcaption>{title}</figcaption>")
            } else {
                String::new()
            };
            let id_attr = if !id.is_empty() {
                format!(" id=\"{id}\"")
            } else {
                String::new()
            };
            format!("<figure class=\"diagram\"{id_attr}>{inner}{caption}</figure>\n")
        }

        Block::Figure {
            id,
            number,
            title,
            content,
            ..
        } => {
            let inner = render_blocks(content, ctx);
            let caption = if !number.is_empty() && !title.is_empty() {
                format!("<figcaption><strong>Figure {number}.</strong> {title}</figcaption>")
            } else if !number.is_empty() {
                format!("<figcaption><strong>Figure {number}.</strong></figcaption>")
            } else if !title.is_empty() {
                format!("<figcaption>{title}</figcaption>")
            } else {
                String::new()
            };
            format!("<figure class=\"labeled-block figure\" id=\"{id}\">{inner}{caption}</figure>\n")
        }

        Block::MathDisplay(latex) => {
            let escaped = html_escape(latex);
            format!("<div class=\"math-display\" data-latex=\"{escaped}\">\\[{escaped}\\]</div>\n")
        }

        Block::Callout { kind, content } => {
            let label = kind.to_uppercase();
            let inner = render_blocks(content, ctx);
            format!(
                "<div class=\"callout callout-{kind}\"><div class=\"callout-title\">{label}</div><div \
                 class=\"callout-body\">{inner}</div></div>\n"
            )
        }

        Block::Blockquote(content) => {
            let inner = render_blocks(content, ctx);
            format!("<blockquote>{inner}</blockquote>\n")
        }

        Block::List { ordered, start, items } => {
            let tag = if *ordered { "ol" } else { "ul" };
            let start_attr = if *ordered && *start != 1 {
                format!(" start=\"{start}\"")
            } else {
                String::new()
            };
            let mut html = format!("<{tag}{start_attr}>\n");
            for item in items {
                // Tight list items: if just a single paragraph, render inlines
                // directly without <p> wrapper to avoid block-level breaks.
                let inner = if item.len() == 1 {
                    if let Block::Paragraph(inlines) = &item[0] {
                        render_inlines(inlines, ctx)
                    } else {
                        render_blocks(item, ctx)
                    }
                } else {
                    render_blocks(item, ctx)
                };
                html.push_str(&format!("<li>{inner}</li>\n"));
            }
            html.push_str(&format!("</{tag}>\n"));
            html
        }

        Block::Table {
            alignments,
            header,
            rows,
        } => {
            let mut html = String::from("<table>\n<thead><tr>\n");
            for (i, cell) in header.iter().enumerate() {
                let align = alignments.get(i).copied().unwrap_or(Alignment::None);
                let style = alignment_style(align);
                let inner = render_inlines(cell, ctx);
                html.push_str(&format!("<th{style}>{inner}</th>\n"));
            }
            html.push_str("</tr></thead>\n<tbody>\n");
            for row in rows {
                html.push_str("<tr>\n");
                for (i, cell) in row.iter().enumerate() {
                    let align = alignments.get(i).copied().unwrap_or(Alignment::None);
                    let style = alignment_style(align);
                    let inner = render_inlines(cell, ctx);
                    html.push_str(&format!("<td{style}>{inner}</td>\n"));
                }
                html.push_str("</tr>\n");
            }
            html.push_str("</tbody></table>\n");
            html
        }

        Block::FootnoteDef { id, content } => {
            let num = ctx.footnote_number(id);
            let inner = render_blocks(content, ctx);
            format!(
                "<div class=\"footnote-definition\" id=\"fn-{id}\"><span class=\"footnote-definition-label\"><a \
                 href=\"#fnref-{id}\">{num}</a></span>{inner}</div>\n"
            )
        }

        Block::TableOfContents(entries) => {
            let min_level = entries.iter().map(|e| e.level).min().unwrap_or(1);
            let mut html = String::from("<nav class=\"post-toc\"><details open><summary>Contents</summary><ul>\n");
            for entry in entries {
                let depth = entry.level.saturating_sub(min_level);
                let num_span = if !entry.number.is_empty() {
                    format!("<span class=\"toc-num\">{}</span>", html_escape(&entry.number))
                } else {
                    String::new()
                };
                let depth_class = match depth {
                    0 => "",
                    1 => " toc-h2",
                    _ => " toc-h3",
                };
                html.push_str(&format!(
                    "<li class=\"toc-entry{depth_class}\"><a href=\"#{}\">{}{}</a></li>\n",
                    entry.id,
                    num_span,
                    html_escape(&entry.title)
                ));
            }
            html.push_str("</ul></details></nav>\n");
            html
        }

        Block::ThematicBreak => "<hr />\n".to_string(),

        Block::Html(raw) => format!("{raw}\n"),
    }
}

// ============================================================
// Inline rendering
// ============================================================

fn render_inlines<F: Fn(&str) -> Option<PostPreview>>(inlines: &[Inline], ctx: &mut RenderContext<F>) -> String {
    let mut html = String::new();
    for inline in inlines {
        html.push_str(&render_inline(inline, ctx));
    }
    return html;
}

fn render_inline<F: Fn(&str) -> Option<PostPreview>>(inline: &Inline, ctx: &mut RenderContext<F>) -> String {
    match inline {
        Inline::Text(text) => html_escape(text),

        Inline::Bold(children) => {
            format!("<strong>{}</strong>", render_inlines(children, ctx))
        }

        Inline::Italic(children) => {
            format!("<em>{}</em>", render_inlines(children, ctx))
        }

        Inline::Strikethrough(children) => {
            format!("<del>{}</del>", render_inlines(children, ctx))
        }

        Inline::Code(code) => {
            format!("<code>{}</code>", html_escape(code))
        }

        Inline::Math(latex) => {
            let escaped = html_escape(latex);
            format!("<span class=\"math-inline\" data-latex=\"{escaped}\">\\({escaped}\\)</span>")
        }

        Inline::Link { url, children } => {
            let inner = render_inlines(children, ctx);
            let is_trackable = url.starts_with("/blog/") || url.starts_with("http");

            if is_trackable {
                let anchor_id = ctx.next_link_id(url);
                let meta_attrs = ctx.post_metadata_attrs(url);
                format!(
                    "<a href=\"{url}\" id=\"{anchor_id}\"{meta_attrs} target=\"_blank\" rel=\"noopener\">{inner}</a>"
                )
            } else {
                format!("<a href=\"{url}\" target=\"_blank\" rel=\"noopener\">{inner}</a>")
            }
        }

        Inline::Image { url, alt, title: _ } => {
            let id = ctx.next_image_id();
            let escaped_alt = html_escape(alt);
            format!(
                "<figure id=\"img-{id}\" class=\"post-figure\"><img src=\"{url}\" alt=\"{escaped_alt}\" \
                 loading=\"lazy\" /></figure>"
            )
        }

        Inline::CrossRef {
            label,
            display,
            kind,
            title,
            number,
            content_preview,
        } => {
            let id = label.replace(':', "-");
            let href = if label.contains('#') {
                let parts: Vec<&str> = label.splitn(2, '#').collect();
                format!("/blog/{}#{}", parts[0], parts[1].replace(':', "-"))
            } else {
                format!("#{id}")
            };

            let display_text = if display.is_empty() { label } else { display };

            let mut attrs = format!(
                "class=\"xref\" href=\"{href}\" data-preview=\"{}\"",
                html_escape(content_preview)
            );
            if !kind.is_empty() {
                attrs.push_str(&format!(
                    " data-block-label=\"{}\" data-block-kind=\"{}\" data-block-title=\"{}\" data-block-number=\"{}\"",
                    html_escape(label),
                    html_escape(kind),
                    html_escape(title),
                    html_escape(number),
                ));
            }

            format!("<a {attrs}>{}</a>", html_escape(display_text))
        }

        Inline::Citation {
            key,
            anchor_id,
            display,
            ..
        } => {
            let cite_id = format!("cite-{key}");
            let id_attr = if !anchor_id.is_empty() {
                format!(" id=\"{anchor_id}\"")
            } else {
                String::new()
            };
            let display_text = if display.is_empty() { key } else { display };
            format!(
                "<a href=\"#{cite_id}\"{id_attr} class=\"citation\">[{}]</a>",
                html_escape(display_text)
            )
        }

        Inline::FootnoteRef(id) => {
            let num = ctx.footnote_number(id);
            format!("<sup class=\"footnote-reference\"><a href=\"#fn-{id}\" id=\"fnref-{id}\">{num}</a></sup>")
        }

        Inline::SoftBreak => "\n".to_string(),
        Inline::HardBreak => "<br />\n".to_string(),
        Inline::Html(raw) => raw.clone(),
    }
}

// ============================================================
// Helpers
// ============================================================

pub fn html_escape(input: &str) -> String {
    return input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
}

fn alignment_style(align: Alignment) -> String {
    match align {
        Alignment::None => String::new(),
        Alignment::Left => " style=\"text-align:left\"".to_string(),
        Alignment::Center => " style=\"text-align:center\"".to_string(),
        Alignment::Right => " style=\"text-align:right\"".to_string(),
    }
}

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
