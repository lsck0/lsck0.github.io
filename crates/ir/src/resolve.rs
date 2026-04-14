#![allow(clippy::needless_return)]

//! Reference resolution: cross-references, citations, and auto-linking.
//!
//! Operates on the IR tree, replacing placeholder `CrossRef` and `Citation`
//! nodes with resolved versions (display text, anchor ids, etc.).

use std::collections::HashMap;

use crate::{bib::BibEntry, types::*};

// ============================================================
// Label registry
// ============================================================

/// Information about a labeled block, used to resolve cross-references.
pub struct LabelInfo {
    pub slug: String,
    pub kind: String,
    pub title: String,
    pub number: String,
    pub content_preview: String,
}

/// Build a label registry from all pages' block metadata.
pub fn build_label_registry(pages: &[Page]) -> HashMap<String, LabelInfo> {
    let mut registry = HashMap::new();

    for page in pages {
        for block in &page.blocks {
            if block.label.is_empty() {
                continue;
            }
            // Skip auto-generated labels for bare diagrams (no explicit {#label})
            if block.label.starts_with("tikz-")
                || block.label.starts_with("tikzcd-")
                || block.label.starts_with("mermaid-")
            {
                continue;
            }
            if let Some(existing) = registry.get(&block.label) {
                let existing: &LabelInfo = existing;
                eprintln!(
                    "WARNING: Duplicate label \"{}\": defined in both \"{}\" and \"{}\". Use a unique {{#label}} to \
                     disambiguate.",
                    block.label, existing.slug, page.slug,
                );
                continue;
            }
            registry.insert(
                block.label.clone(),
                LabelInfo {
                    slug: page.slug.clone(),
                    kind: block.kind.clone(),
                    title: block.title.clone(),
                    number: block.number.clone(),
                    content_preview: block.content_preview.clone(),
                },
            );
        }
    }

    return registry;
}

// ============================================================
// Cross-reference resolution
// ============================================================

/// Resolve all `CrossRef` nodes in a page's content tree.
pub fn resolve_cross_references(content: &mut [Block], current_slug: &str, registry: &HashMap<String, LabelInfo>) {
    for block in content.iter_mut() {
        resolve_block_cross_refs(block, current_slug, registry);
    }
}

fn resolve_block_cross_refs(block: &mut Block, current_slug: &str, registry: &HashMap<String, LabelInfo>) {
    match block {
        Block::Heading { children, .. } | Block::Paragraph(children) => {
            resolve_inlines_cross_refs(children, current_slug, registry);
        }
        Block::LabeledBlock { content, .. }
        | Block::Figure { content, .. }
        | Block::Callout { content, .. }
        | Block::Blockquote(content) => {
            resolve_cross_references(content, current_slug, registry);
        }
        Block::List { items, .. } => {
            for item in items.iter_mut() {
                resolve_cross_references(item, current_slug, registry);
            }
        }
        Block::Table { header, rows, .. } => {
            for cell in header.iter_mut() {
                resolve_inlines_cross_refs(cell, current_slug, registry);
            }
            for row in rows.iter_mut() {
                for cell in row.iter_mut() {
                    resolve_inlines_cross_refs(cell, current_slug, registry);
                }
            }
        }
        Block::FootnoteDef { content, .. } => {
            resolve_cross_references(content, current_slug, registry);
        }
        _ => {}
    }
}

fn resolve_inlines_cross_refs(inlines: &mut [Inline], current_slug: &str, registry: &HashMap<String, LabelInfo>) {
    for inline in inlines.iter_mut() {
        match inline {
            Inline::CrossRef {
                label,
                display,
                kind,
                title,
                number,
                content_preview,
            } => {
                // Support [[slug#label]] for cross-post references
                let (_, effective_label) = if let Some(hash) = label.find('#') {
                    (Some(&label[..hash]), label[hash + 1..].to_string())
                } else {
                    (None, label.clone())
                };

                if let Some(info) = registry.get(&effective_label) {
                    if display.is_empty() {
                        *display = default_xref_display(&info.kind, &info.title, &info.number);
                    }
                    *kind = info.kind.clone();
                    *title = info.title.clone();
                    *number = info.number.clone();
                    *content_preview = info.content_preview.clone();
                } else if display.is_empty() {
                    eprintln!("WARNING: broken cross-reference [[{label}]] in post \"{current_slug}\".");
                    *display = format!("[[{label}]]");
                }
            }
            Inline::Bold(children) | Inline::Italic(children) | Inline::Strikethrough(children) => {
                resolve_inlines_cross_refs(children, current_slug, registry);
            }
            Inline::Link { children, .. } => {
                resolve_inlines_cross_refs(children, current_slug, registry);
            }
            _ => {}
        }
    }
}

/// Generate default display text for a cross-reference.
fn default_xref_display(kind: &str, title: &str, number: &str) -> String {
    match kind {
        "equation" => {
            if !number.is_empty() {
                return format!("Equation {number}");
            }
            return "Equation".to_string();
        }
        "figure" => {
            if !number.is_empty() {
                return format!("Figure {number}");
            }
            return "Figure".to_string();
        }
        "tikz" | "tikzcd" | "mermaid" => {
            if !title.is_empty() {
                return title.to_string();
            }
            return "Diagram".to_string();
        }
        "proof" => {
            if !title.is_empty() {
                return format!("Proof ({title})");
            }
            return "Proof".to_string();
        }
        _ => {}
    }
    if !title.is_empty() {
        return title.to_string();
    }
    let kind_display = crate::capitalize(kind);
    if !number.is_empty() {
        format!("{kind_display} {number}")
    } else {
        kind_display
    }
}

// ============================================================
// Citation resolution
// ============================================================

/// Resolve all `Citation` nodes: assign anchor ids, populate CitationMeta.
pub fn resolve_citations(
    content: &mut [Block],
    bibliography: &HashMap<String, BibEntry>,
    slug: &str,
) -> Vec<CitationMeta> {
    let mut cite_counter: u32 = 0;
    let mut cited_keys: Vec<String> = Vec::new();
    let mut backlinks: HashMap<String, Vec<String>> = HashMap::new();

    resolve_citations_in_blocks(
        content,
        bibliography,
        slug,
        &mut cite_counter,
        &mut cited_keys,
        &mut backlinks,
    );

    // Build citation metadata
    let mut metas = Vec::new();
    for key in &cited_keys {
        if let Some(entry) = bibliography.get(key) {
            metas.push(CitationMeta {
                key: key.clone(),
                label: entry.citation_label(),
                formatted_html: entry.format_html(),
                backlink_ids: backlinks.get(key).cloned().unwrap_or_default(),
            });
        }
    }

    return metas;
}

fn resolve_citations_in_blocks(
    blocks: &mut [Block],
    bibliography: &HashMap<String, BibEntry>,
    slug: &str,
    counter: &mut u32,
    cited_keys: &mut Vec<String>,
    backlinks: &mut HashMap<String, Vec<String>>,
) {
    for block in blocks.iter_mut() {
        match block {
            Block::Heading { children, .. } | Block::Paragraph(children) => {
                resolve_citations_in_inlines(children, bibliography, slug, counter, cited_keys, backlinks);
            }
            Block::LabeledBlock { content, .. }
            | Block::Figure { content, .. }
            | Block::Callout { content, .. }
            | Block::Blockquote(content) => {
                resolve_citations_in_blocks(content, bibliography, slug, counter, cited_keys, backlinks);
            }
            Block::List { items, .. } => {
                for item in items.iter_mut() {
                    resolve_citations_in_blocks(item, bibliography, slug, counter, cited_keys, backlinks);
                }
            }
            Block::Table { header, rows, .. } => {
                for cell in header.iter_mut() {
                    resolve_citations_in_inlines(cell, bibliography, slug, counter, cited_keys, backlinks);
                }
                for row in rows.iter_mut() {
                    for cell in row.iter_mut() {
                        resolve_citations_in_inlines(cell, bibliography, slug, counter, cited_keys, backlinks);
                    }
                }
            }
            Block::FootnoteDef { content, .. } => {
                resolve_citations_in_blocks(content, bibliography, slug, counter, cited_keys, backlinks);
            }
            _ => {}
        }
    }
}

fn resolve_citations_in_inlines(
    inlines: &mut [Inline],
    bibliography: &HashMap<String, BibEntry>,
    slug: &str,
    counter: &mut u32,
    cited_keys: &mut Vec<String>,
    backlinks: &mut HashMap<String, Vec<String>>,
) {
    for inline in inlines.iter_mut() {
        match inline {
            Inline::Citation {
                key,
                locator,
                anchor_id,
                display,
            } => {
                if let Some(entry) = bibliography.get(key.as_str()) {
                    // Pre-compute display text: "key" or "key, locator"
                    let label = entry.citation_label();
                    *display = if !locator.is_empty() {
                        format!("{label}, {locator}")
                    } else {
                        label
                    };
                } else {
                    eprintln!("WARNING: unknown citation key [@{key}] in post \"{slug}\".");
                    *display = format!("?{key}");
                }

                // Track cited keys in order
                if !cited_keys.contains(key) {
                    cited_keys.push(key.clone());
                }

                // Assign unique anchor id
                *counter += 1;
                *anchor_id = format!("cite-ref-{counter}");

                // Track backlinks
                backlinks.entry(key.clone()).or_default().push(anchor_id.clone());
            }
            Inline::Bold(children) | Inline::Italic(children) | Inline::Strikethrough(children) => {
                resolve_citations_in_inlines(children, bibliography, slug, counter, cited_keys, backlinks);
            }
            Inline::Link { children, .. } => {
                resolve_citations_in_inlines(children, bibliography, slug, counter, cited_keys, backlinks);
            }
            _ => {}
        }
    }
}

// ============================================================
// Auto-linking
// ============================================================

struct TermEntry {
    title_lower: String,
    label: String,
}

/// Auto-link definition terms in prose text.
/// Scans `Text` nodes for known definition titles and wraps them in `CrossRef`.
pub fn auto_link_definitions(content: &mut [Block], registry: &HashMap<String, LabelInfo>) {
    // Build term index: sorted longest-first for greedy matching
    let mut terms: Vec<TermEntry> = Vec::new();
    for (label, info) in registry {
        if info.title.is_empty() || info.kind == "proof" || info.kind == "example" || info.kind == "equation" {
            continue;
        }

        // Primary title
        let title_lower = info.title.to_lowercase();
        terms.push(TermEntry {
            title_lower: title_lower.clone(),
            label: label.clone(),
        });

        // Auto-plural
        let plural = if title_lower.ends_with('s')
            || title_lower.ends_with('x')
            || title_lower.ends_with('z')
            || title_lower.ends_with("ch")
            || title_lower.ends_with("sh")
        {
            format!("{title_lower}es")
        } else {
            format!("{title_lower}s")
        };
        terms.push(TermEntry {
            title_lower: plural,
            label: label.clone(),
        });
    }

    // Also add aliases
    // (aliases are stored in BlockMeta — we'd need to pass them through. For now skip aliases.)

    terms.sort_by_key(|t| std::cmp::Reverse(t.title_lower.len()));

    for block in content.iter_mut() {
        auto_link_block(block, &terms, registry);
    }
}

fn auto_link_block(block: &mut Block, terms: &[TermEntry], registry: &HashMap<String, LabelInfo>) {
    match block {
        Block::Paragraph(children) => {
            *children = auto_link_inlines(std::mem::take(children), terms, registry);
        }
        Block::LabeledBlock { content, .. }
        | Block::Figure { content, .. }
        | Block::Callout { content, .. }
        | Block::Blockquote(content) => {
            for b in content.iter_mut() {
                auto_link_block(b, terms, registry);
            }
        }
        Block::List { items, .. } => {
            for item in items.iter_mut() {
                for b in item.iter_mut() {
                    auto_link_block(b, terms, registry);
                }
            }
        }
        Block::FootnoteDef { content, .. } => {
            for b in content.iter_mut() {
                auto_link_block(b, terms, registry);
            }
        }
        _ => {}
    }
}

fn auto_link_inlines(inlines: Vec<Inline>, terms: &[TermEntry], registry: &HashMap<String, LabelInfo>) -> Vec<Inline> {
    let mut result = Vec::new();

    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                let text_lower = text.to_lowercase();
                let mut pos = 0;

                while pos < text.len() {
                    if !is_word_start(&text, pos) {
                        let ch = text[pos..].chars().next().unwrap();
                        result.push(Inline::Text(ch.to_string()));
                        pos += ch.len_utf8();
                        continue;
                    }

                    let mut matched = false;
                    for term in terms {
                        let end = pos + term.title_lower.len();
                        if end > text.len() {
                            continue;
                        }
                        if !text.is_char_boundary(end) || !text_lower.is_char_boundary(end) {
                            continue;
                        }
                        if text_lower[pos..end] == term.title_lower && is_word_end(&text, end) {
                            // Emit text before match
                            // Emit CrossRef
                            let original = &text[pos..end];
                            let info = registry.get(&term.label);
                            result.push(Inline::CrossRef {
                                label: term.label.clone(),
                                display: original.to_string(),
                                kind: info.map(|i| i.kind.clone()).unwrap_or_default(),
                                title: info.map(|i| i.title.clone()).unwrap_or_default(),
                                number: info.map(|i| i.number.clone()).unwrap_or_default(),
                                content_preview: info.map(|i| i.content_preview.clone()).unwrap_or_default(),
                            });
                            pos = end;
                            matched = true;
                            break;
                        }
                    }

                    if !matched {
                        let ch = text[pos..].chars().next().unwrap();
                        result.push(Inline::Text(ch.to_string()));
                        pos += ch.len_utf8();
                    }
                }
            }
            Inline::Bold(children) => {
                result.push(Inline::Bold(auto_link_inlines(children, terms, registry)));
            }
            Inline::Italic(children) => {
                result.push(Inline::Italic(auto_link_inlines(children, terms, registry)));
            }
            other => result.push(other),
        }
    }

    // Consolidate adjacent Text nodes
    return consolidate_text_nodes(result);
}

fn consolidate_text_nodes(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut result: Vec<Inline> = Vec::new();
    for inline in inlines {
        if let Inline::Text(text) = &inline
            && let Some(Inline::Text(prev)) = result.last_mut()
        {
            prev.push_str(text);
            continue;
        }
        result.push(inline);
    }
    return result;
}

fn is_word_start(text: &str, pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    let prev = text[..pos].chars().next_back().unwrap();
    return !prev.is_alphanumeric() && prev != '_';
}

fn is_word_end(text: &str, pos: usize) -> bool {
    if pos >= text.len() {
        return true;
    }
    let next = text[pos..].chars().next().unwrap();
    return !next.is_alphanumeric() && next != '_';
}

// ============================================================
// Link extraction
// ============================================================

/// Extract internal and external links from a page's content tree.
/// `current_slug` identifies the current page; `registry` maps labels to their
/// owning slug so bare `[[label]]` refs to other posts are tracked as internal links.
pub fn extract_links(
    content: &[Block],
    current_slug: &str,
    registry: &HashMap<String, LabelInfo>,
) -> (Vec<String>, Vec<String>) {
    let mut internal = Vec::new();
    let mut external = Vec::new();

    for block in content {
        extract_links_from_block(block, current_slug, registry, &mut internal, &mut external);
    }

    return (internal, external);
}

fn extract_links_from_block(
    block: &Block,
    current_slug: &str,
    registry: &HashMap<String, LabelInfo>,
    internal: &mut Vec<String>,
    external: &mut Vec<String>,
) {
    match block {
        Block::Heading { children, .. } | Block::Paragraph(children) => {
            extract_links_from_inlines(children, current_slug, registry, internal, external);
        }
        Block::LabeledBlock { content, .. }
        | Block::Figure { content, .. }
        | Block::Callout { content, .. }
        | Block::Blockquote(content) => {
            for b in content {
                extract_links_from_block(b, current_slug, registry, internal, external);
            }
        }
        Block::List { items, .. } => {
            for item in items {
                for b in item {
                    extract_links_from_block(b, current_slug, registry, internal, external);
                }
            }
        }
        Block::FootnoteDef { content, .. } => {
            for b in content {
                extract_links_from_block(b, current_slug, registry, internal, external);
            }
        }
        _ => {}
    }
}

fn extract_links_from_inlines(
    inlines: &[Inline],
    current_slug: &str,
    registry: &HashMap<String, LabelInfo>,
    internal: &mut Vec<String>,
    external: &mut Vec<String>,
) {
    for inline in inlines {
        match inline {
            Inline::Link { url, children, .. } => {
                if let Some(path) = url.strip_prefix("/blog/") {
                    let slug = path.split('#').next().unwrap_or("");
                    if !slug.is_empty() && !internal.contains(&slug.to_string()) {
                        internal.push(slug.to_string());
                    }
                } else if url.starts_with("http") && !external.contains(url) {
                    external.push(url.clone());
                }
                extract_links_from_inlines(children, current_slug, registry, internal, external);
            }
            Inline::CrossRef { label, .. } => {
                // [[slug#label]] — explicit cross-post reference
                if let Some(hash) = label.find('#') {
                    let slug = &label[..hash];
                    if !slug.is_empty() && !internal.contains(&slug.to_string()) {
                        internal.push(slug.to_string());
                    }
                } else if let Some(info) = registry.get(label) {
                    // bare [[label]] that resolves to a different post
                    if info.slug != current_slug && !internal.contains(&info.slug) {
                        internal.push(info.slug.clone());
                    }
                }
            }
            Inline::Bold(c) | Inline::Italic(c) | Inline::Strikethrough(c) => {
                extract_links_from_inlines(c, current_slug, registry, internal, external);
            }
            _ => {}
        }
    }
}

// ============================================================
// Helpers
// ============================================================
