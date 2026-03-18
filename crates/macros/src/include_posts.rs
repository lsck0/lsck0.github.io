use std::{collections::HashMap, env, path::PathBuf};

use content::{
    ParsedPost, extract_external_links, extract_internal_links, parse_posts_directory, resolve_transclusions,
};
use proc_macro::TokenStream;
use quote::quote;

// ============================================================
// Labeled block kinds
// ============================================================

const NUMBERED_KINDS: &[&str] = &[
    "definition",
    "theorem",
    "lemma",
    "corollary",
    "proposition",
    "example",
    "axiom",
    "remark",
    "conjecture",
    "exercise",
    "problem",
];
const UNNUMBERED_KINDS: &[&str] = &["proof"];
const CALLOUT_KINDS: &[&str] = &["tip", "warning", "danger", "note", "info"];

// ============================================================
// Labeled block data
// ============================================================

struct LabeledBlock {
    label: String,
    kind: String,
    title: String,
    aliases: Vec<String>,
    number: u32,
    content: String,
}

// ============================================================
// Entry point
// ============================================================

pub fn include_posts_impl(_input: TokenStream) -> TokenStream {
    let manifest_directory = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let content_directory = PathBuf::from(&manifest_directory).join("content/posts");
    if !content_directory.exists() {
        panic!("Content directory not found at {}.", content_directory.display());
    }

    let mut posts = parse_posts_directory(&content_directory);
    resolve_transclusions(&mut posts);

    // Extract labeled blocks from all posts
    let mut blocks_per_post: Vec<Vec<LabeledBlock>> = Vec::new();
    for post in &mut posts {
        let (new_body, blocks) = extract_labeled_blocks(&post.body);
        post.body = new_body;
        blocks_per_post.push(blocks);
    }

    // Build global label registry: label -> (slug, kind, title, number, content)
    let registry: HashMap<String, (String, String, String, u32, String)> = posts
        .iter()
        .zip(blocks_per_post.iter())
        .flat_map(|(post, blocks)| {
            blocks
                .iter()
                .map(|block| {
                    (
                        block.label.clone(),
                        (
                            post.slug.clone(),
                            block.kind.clone(),
                            block.title.clone(),
                            block.number,
                            block.content.clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Build alias map: label -> aliases
    let alias_map: HashMap<String, Vec<String>> = blocks_per_post
        .iter()
        .flat_map(|blocks| blocks.iter().map(|b| (b.label.clone(), b.aliases.clone())))
        .collect();

    // Resolve cross-references in block preview content first
    let mut resolved_registry = registry.clone();
    for (label, (_slug, _kind, _title, _number, content)) in resolved_registry.iter_mut() {
        *content = resolve_cross_references_preview(content, label, &registry);
    }

    // Resolve explicit [[label]] cross-references in post bodies
    for post in &mut posts {
        post.body = resolve_cross_references(&post.body, &post.slug, &resolved_registry);
    }

    // Auto-link defined terms in prose (after explicit xrefs are resolved)
    let term_index = build_term_index(&resolved_registry, &alias_map);
    for post in &mut posts {
        post.body = auto_link_definitions(&post.body, &post.slug, &term_index, &resolved_registry);
    }

    // Re-extract links since cross-refs and auto-links may add internal links
    for post in &mut posts {
        post.internal_links = extract_internal_links(&post.body);
        post.external_links = extract_external_links(&post.body);
    }

    // Sort by date descending
    posts.sort_by(|a, b| b.date().cmp(a.date()));

    // Filter out drafts in release mode; keep them in dev mode
    #[cfg(not(debug_assertions))]
    posts.retain(|post| !post.is_draft());

    let post_tokens = posts
        .iter()
        .zip(blocks_per_post.iter())
        .map(|(post, blocks)| emit_post_tokens(post, blocks));

    let output = quote! {
        &[#(#post_tokens),*]
    };

    return output.into();
}

// ============================================================
// Token emission
// ============================================================

fn emit_post_tokens(post: &ParsedPost, blocks: &[LabeledBlock]) -> proc_macro2::TokenStream {
    let slug = &post.slug;
    let folder = &post.folder;
    let body = &post.body;

    let metadata_entries = post.metadata.iter().map(|(key, value)| {
        quote! { (#key, #value) }
    });

    let internal_link_entries = post.internal_links.iter().map(|link| {
        quote! { #link }
    });

    let external_link_entries = post.external_links.iter().map(|link| {
        quote! { #link }
    });

    let source_entries: Vec<String> = post.sources().iter().map(|s| s.to_string()).collect();
    let source_tokens = source_entries.iter().map(|source| {
        quote! { #source }
    });

    let block_tokens = blocks.iter().map(|block| {
        let label = &block.label;
        let kind = &block.kind;
        let title = &block.title;
        let number = block.number;
        let content = &block.content;

        quote! {
            LabeledBlock {
                label: #label,
                kind: #kind,
                title: #title,
                number: #number,
                content: #content,
            }
        }
    });

    return quote! {
        Post {
            slug: #slug,
            folder: #folder,
            metadata: &[#(#metadata_entries),*],
            body: #body,
            internal_links: &[#(#internal_link_entries),*],
            external_links: &[#(#external_link_entries),*],
            sources: &[#(#source_tokens),*],
            labeled_blocks: &[#(#block_tokens),*],
        }
    };
}

// ============================================================
// Labeled blocks (```kind Title {#label} ... ```)
// ============================================================

fn extract_labeled_blocks(body: &str) -> (String, Vec<LabeledBlock>) {
    let all_kinds: Vec<&str> = NUMBERED_KINDS
        .iter()
        .chain(UNNUMBERED_KINDS.iter())
        .chain(CALLOUT_KINDS.iter())
        .copied()
        .collect();
    let mut blocks = Vec::new();
    let mut result = String::new();
    let mut counter: u32 = 0;
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Check for backtick-fenced block: ```kind or ````kind etc.
        if let Some((fence_len, rest)) = parse_backtick_fence(trimmed)
            && let Some((kind, title, aliases, label_opt)) = parse_block_header(rest)
            && all_kinds.contains(&kind.as_str())
        {
            let is_callout = CALLOUT_KINDS.contains(&kind.as_str());
            let is_numbered = NUMBERED_KINDS.contains(&kind.as_str());
            let number = if is_numbered {
                counter += 1;
                counter
            } else {
                0
            };

            let label = label_opt
                .unwrap_or_else(|| format!("{}-{}", kind, if is_numbered { number } else { blocks.len() as u32 }));

            // Collect content until matching closing fence (same or more backticks)
            i += 1;
            let mut content = String::new();
            while i < lines.len() {
                let line_trimmed = lines[i].trim();
                if is_closing_fence(line_trimmed, fence_len) {
                    break;
                }
                if !content.is_empty() {
                    content.push('\n');
                }
                content.push_str(lines[i]);
                i += 1;
            }
            let content = content.trim().to_string();

            if is_callout {
                // Emit callout HTML directly
                result.push_str(&format!(
                    "<div class=\"callout callout-{kind}\">\n<div class=\"callout-title\">{}</div>\n<div \
                     class=\"callout-body\">\n\n{content}\n\n</div>\n</div>\n",
                    capitalize_first(&kind)
                ));
            } else {
                blocks.push(LabeledBlock {
                    label: label.clone(),
                    kind: kind.clone(),
                    title: title.clone(),
                    aliases,
                    number,
                    content: content.clone(),
                });

                // Emit HTML comment markers (pulldown-cmark passes these through)
                let html_id = label.replace(':', "-");
                let escaped_title = title.replace('|', "\\|");
                result.push_str(&format!(
                    "<!--BLOCK|{}|{}|{}|{}-->\n\n{}\n\n<!--/BLOCK-->\n",
                    kind, html_id, number, escaped_title, content
                ));
            }

            i += 1; // skip closing fence
            continue;
        }

        result.push_str(lines[i]);
        result.push('\n');
        i += 1;
    }

    return (result, blocks);
}

/// Parse a backtick fence opening line. Returns (fence_length, rest_after_backticks) if the line
/// starts with 3+ backticks followed by a non-empty "language" tag.
fn parse_backtick_fence(line: &str) -> Option<(usize, &str)> {
    let backtick_count = line.chars().take_while(|&c| c == '`').count();
    if backtick_count < 3 {
        return None;
    }
    let rest = line[backtick_count..].trim();
    if rest.is_empty() {
        return None;
    }
    return Some((backtick_count, rest));
}

/// Check if a line is a closing fence with at least `min_len` backticks and nothing else.
fn is_closing_fence(line: &str, min_len: usize) -> bool {
    let backtick_count = line.chars().take_while(|&c| c == '`').count();
    return backtick_count >= min_len && line[backtick_count..].trim().is_empty();
}

/// Parse `kind Title | alias1, alias2 {#label}` from the text after backticks.
/// Returns (kind, title, aliases, optional_label).
fn parse_block_header(header: &str) -> Option<(String, String, Vec<String>, Option<String>)> {
    let kind_end = header.find(|c: char| c.is_whitespace()).unwrap_or(header.len());
    let kind = header[..kind_end].to_lowercase();
    let after_kind = header[kind_end..].trim();

    let (title_and_aliases, label) = if let Some(label_start) = after_kind.rfind("{#") {
        let after = &after_kind[label_start + 2..];
        if let Some(label_end) = after.find('}') {
            let l = after[..label_end].to_string();
            let before = after_kind[..label_start].trim().to_string();
            (before, Some(l))
        } else {
            (after_kind.to_string(), None)
        }
    } else {
        (after_kind.to_string(), None)
    };

    // Split title from aliases on `|`
    let (title, aliases) = if let Some(pipe_pos) = title_and_aliases.find('|') {
        let title = title_and_aliases[..pipe_pos].trim().to_string();
        let alias_str = &title_and_aliases[pipe_pos + 1..];
        let aliases: Vec<String> = alias_str
            .split(',')
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .collect();
        (title, aliases)
    } else {
        (title_and_aliases, Vec::new())
    };

    return Some((kind, title, aliases, label));
}

// ============================================================
// Cross-reference resolution ([[label]] → HTML links)
// ============================================================

fn resolve_cross_references(
    body: &str,
    current_slug: &str,
    registry: &HashMap<String, (String, String, String, u32, String)>,
) -> String {
    let mut result = String::new();
    let mut remaining = body;

    while let Some(start) = remaining.find("[[") {
        // Skip transclusion markers ![[
        if start > 0 && remaining.as_bytes()[start - 1] == b'!' {
            result.push_str(&remaining[..start + 2]);
            remaining = &remaining[start + 2..];
            continue;
        }

        result.push_str(&remaining[..start]);
        let after = &remaining[start + 2..];

        if let Some(end) = after.find("]]") {
            let ref_content = &after[..end];

            let (ref_key, custom_display) = if let Some(pipe) = ref_content.find('|') {
                (&ref_content[..pipe], Some(&ref_content[pipe + 1..]))
            } else {
                (ref_content, None)
            };

            // Support [[slug#label]] for cross-post references
            let (target_slug_override, label) = if let Some(hash) = ref_key.find('#') {
                (Some(&ref_key[..hash]), &ref_key[hash + 1..])
            } else {
                (None, ref_key)
            };

            if let Some((slug, kind, title, number, content)) = registry.get(label) {
                let html_id = label.replace(':', "-");
                let effective_slug = target_slug_override.unwrap_or(slug.as_str());
                let href = if effective_slug == current_slug {
                    format!("#{html_id}")
                } else {
                    format!("/blog/{effective_slug}#{html_id}")
                };

                let display_text = if let Some(custom) = custom_display {
                    custom.to_string()
                } else {
                    default_xref_display(kind, title, *number)
                };

                let escaped_preview = escape_for_attribute(content);

                result.push_str(&format!(
                    "<a class=\"xref\" href=\"{href}\" data-preview=\"{escaped_preview}\">{display_text}</a>"
                ));
            } else {
                result.push_str(&format!(
                    "<span class=\"xref-broken\" title=\"Unknown reference: {label}\">[[{ref_content}]]</span>"
                ));
            }

            remaining = &after[end + 2..];
        } else {
            result.push_str("[[");
            remaining = after;
        }
    }
    result.push_str(remaining);

    return result;
}

fn resolve_cross_references_preview(
    content: &str,
    _self_label: &str,
    registry: &HashMap<String, (String, String, String, u32, String)>,
) -> String {
    let mut result = String::new();
    let mut remaining = content;

    while let Some(start) = remaining.find("[[") {
        result.push_str(&remaining[..start]);
        let after = &remaining[start + 2..];

        if let Some(end) = after.find("]]") {
            let ref_content = &after[..end];
            let (label, custom_display) = if let Some(pipe) = ref_content.find('|') {
                (&ref_content[..pipe], Some(&ref_content[pipe + 1..]))
            } else {
                (ref_content, None)
            };

            if let Some((_slug, kind, title, number, _content)) = registry.get(label) {
                let display = if let Some(custom) = custom_display {
                    custom.to_string()
                } else {
                    default_xref_display(kind, title, *number)
                };
                result.push_str(&display);
            } else {
                result.push_str(ref_content);
            }
            remaining = &after[end + 2..];
        } else {
            result.push_str("[[");
            remaining = after;
        }
    }
    result.push_str(remaining);
    return result;
}

// ============================================================
// Auto-definition linking
// ============================================================

/// A term from the definition registry that can be auto-linked in prose.
struct TermEntry {
    title_lower: String,
    label: String,
    slug: String,
    #[allow(dead_code)]
    kind: String,
    #[allow(dead_code)]
    title: String,
    preview: String,
}

/// Build a sorted index of linkable terms from the registry.
/// Terms are sorted longest-first so "normal subgroup" matches before "subgroup".
/// Includes extra entries for definition aliases.
fn build_term_index(
    registry: &HashMap<String, (String, String, String, u32, String)>,
    alias_map: &HashMap<String, Vec<String>>,
) -> Vec<TermEntry> {
    let mut terms: Vec<TermEntry> = registry
        .iter()
        .filter(|(_, (_, kind, title, _, _))| !title.is_empty() && kind != "proof" && kind != "example")
        .flat_map(|(label, (slug, kind, title, _number, content))| {
            let preview = escape_for_attribute(content);
            let title_lower = title.to_lowercase();
            let mut entries = vec![TermEntry {
                title_lower: title_lower.clone(),
                label: label.clone(),
                slug: slug.clone(),
                kind: kind.clone(),
                title: title.clone(),
                preview: preview.clone(),
            }];

            // Add automatic plural as alias
            let plural = if title_lower.ends_with('s')
                || title_lower.ends_with('x')
                || title_lower.ends_with('z')
                || title_lower.ends_with("ch")
                || title_lower.ends_with("sh")
            {
                format!("{}es", title_lower)
            } else {
                format!("{}s", title_lower)
            };
            entries.push(TermEntry {
                title_lower: plural,
                label: label.clone(),
                slug: slug.clone(),
                kind: kind.clone(),
                title: format!("{}s", title),
                preview: preview.clone(),
            });

            // Add alias entries that link to the same definition
            if let Some(aliases) = alias_map.get(label) {
                for alias in aliases {
                    entries.push(TermEntry {
                        title_lower: alias.to_lowercase(),
                        label: label.clone(),
                        slug: slug.clone(),
                        kind: kind.clone(),
                        title: alias.clone(),
                        preview: preview.clone(),
                    });
                }
            }

            entries
        })
        .collect();

    // Sort longest first — greedy matching prevents "subgroup" from eating "normal subgroup"
    terms.sort_by_key(|t| std::cmp::Reverse(t.title_lower.len()));
    return terms;
}

/// Auto-link defined terms in a post's body text.
/// Only the first occurrence of each term per post is linked.
/// Skips: HTML tags, comments, inline code, math, and existing xref/auto-def spans.
fn auto_link_definitions(
    body: &str,
    current_slug: &str,
    terms: &[TermEntry],
    _registry: &HashMap<String, (String, String, String, u32, String)>,
) -> String {
    if terms.is_empty() {
        return body.to_string();
    }

    let body_lower = body.to_lowercase();
    let mut result = String::new();
    let mut pos = 0;

    while pos < body.len() {
        // Try to skip opaque regions (HTML tags, comments, code, math)
        if let Some(skip) = skip_opaque_region(body, pos) {
            result.push_str(&body[pos..pos + skip]);
            pos += skip;
            continue;
        }

        // Check if current position is a word boundary (start of a potential term)
        if is_word_start(body, pos) {
            let mut matched = false;
            for term in terms {
                let end = pos + term.title_lower.len();
                if end > body.len() {
                    continue;
                }
                if end > body_lower.len() || !body_lower.is_char_boundary(end) || !body.is_char_boundary(end) {
                    continue;
                }
                if body_lower[pos..end] == term.title_lower && is_word_end(body, end) {
                    // Link every occurrence, not just first
                    // if linked.contains(&term.label) {
                    //     continue;
                    // }

                    let original = &body[pos..end];
                    let html_id = term.label.replace(':', "-");
                    let href = if term.slug == current_slug {
                        format!("#{html_id}")
                    } else {
                        format!("/blog/{}#{html_id}", term.slug)
                    };
                    result.push_str(&format!(
                        "<a class=\"auto-def\" href=\"{href}\" data-preview=\"{}\">{original}</a>",
                        term.preview
                    ));
                    pos = end;
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
        }

        // Advance one character (handle multi-byte UTF-8)
        let ch = body[pos..].chars().next().unwrap();
        result.push(ch);
        pos += ch.len_utf8();
    }

    return result;
}

/// Returns the length of an opaque region starting at `pos`, or None if not in one.
fn skip_opaque_region(body: &str, pos: usize) -> Option<usize> {
    let remaining = &body[pos..];

    // HTML comments (block markers)
    if remaining.starts_with("<!--") {
        return remaining.find("-->").map(|end| end + 3);
    }

    // Existing xref or auto-def tags: skip entire <a class="xref"...>...</a> or <a class="auto-def"...>...</a>
    if remaining.starts_with("<a ")
        && (remaining.contains("class=\"xref\"") || remaining.contains("class=\"auto-def\""))
        && let Some(close) = remaining.find("</a>")
    {
        return Some(close + 4);
    }

    // Any other HTML tag (skip the tag itself, not its content)
    if remaining.starts_with('<') && !remaining.starts_with("<!--") {
        return remaining.find('>').map(|end| end + 1);
    }

    // Fenced code blocks (```)
    if let Some(after) = remaining.strip_prefix("```") {
        return after.find("```").map(|end| end + 6);
    }

    // Inline code
    if let Some(after) = remaining.strip_prefix('`') {
        return after.find('`').map(|end| end + 2);
    }

    // Display math $$...$$
    if let Some(after) = remaining.strip_prefix("$$") {
        return after.find("$$").map(|end| end + 4);
    }

    // Inline math $...$
    if remaining.starts_with('$') && remaining.len() > 1 && remaining.as_bytes()[1] != b'$' {
        let after = &remaining[1..];
        return after.find('$').map(|end| end + 2);
    }

    return None;
}

/// Check if position is at a word boundary suitable for a term start.
fn is_word_start(body: &str, pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    let prev = body[..pos].chars().next_back().unwrap();
    return !prev.is_alphanumeric() && prev != '_';
}

/// Check if position is at a word boundary suitable for a term end.
fn is_word_end(body: &str, pos: usize) -> bool {
    if pos >= body.len() {
        return true;
    }
    let next = body[pos..].chars().next().unwrap();
    return !next.is_alphanumeric() && next != '_';
}

// ============================================================
// Shared helpers
// ============================================================

fn escape_for_attribute(content: &str) -> String {
    return content
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', " ");
}

fn default_xref_display(kind: &str, title: &str, number: u32) -> String {
    if !title.is_empty() {
        return title.to_string();
    }
    let kind_display = capitalize_first(kind);
    if number > 0 {
        format!("{kind_display} {number}")
    } else {
        kind_display
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
