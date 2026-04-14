#![allow(clippy::needless_return)]

//! Proc macro that reads all markdown posts from `content/posts/`, parses them
//! into IR, resolves cross-references and citations, then serializes the result
//! as a `postcard`-encoded byte blob embedded in the binary.
//!
//! At runtime the frontend deserializes once and renders from the IR.

use std::{collections::HashMap, env, fs, path::PathBuf};

use ir::{bib, frontmatter::parse_frontmatter, parse::parse_markdown, resolve, types::*};
use proc_macro::TokenStream;
use quote::quote;

// ============================================================
// Entry point
// ============================================================

pub fn include_posts_impl(_input: TokenStream) -> TokenStream {
    let content_dir = content_dir();

    // ---- Read all markdown files ----
    let mut pages = read_posts(&content_dir.join("posts"));

    // ---- Read bibliography ----
    let bib_path = content_dir.join("references.bib");
    let bibliography = if bib_path.exists() {
        bib::parse_bib_file(&bib_path)
    } else {
        HashMap::new()
    };

    // ---- Parse markdown into IR ----
    for page in &mut pages {
        let has_toc = page.metadata.get("toc").is_some_and(|v| v == "true");
        let (content, block_metas) = parse_markdown(page.metadata.get("body").unwrap_or(&String::new()), has_toc);
        page.content = content;
        page.blocks = block_metas;

        // Extract sources from metadata
        if let Some(sources_str) = page.metadata.get("sources") {
            page.sources = sources_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    // ---- Build label registry (all blocks across all pages) ----
    let registry = resolve::build_label_registry(&pages);

    // ---- Resolve references ----
    for page in &mut pages {
        let slug = page.slug.clone();

        // Cross-references
        resolve::resolve_cross_references(&mut page.content, &slug, &registry);

        // Citations
        let citation_metas = resolve::resolve_citations(&mut page.content, &bibliography, &slug);
        page.citations = citation_metas;

        // Auto-link definitions
        resolve::auto_link_definitions(&mut page.content, &registry);

        // Extract internal/external links (registry detects bare cross-refs to other posts)
        let (internal, external) = resolve::extract_links(&page.content, &slug, &registry);
        page.internal_links = internal;
        page.external_links = external;
    }

    // ---- Remove the raw body from metadata (it's now in the IR) ----
    for page in &mut pages {
        page.metadata.remove("body");
    }

    // ---- Generate table of contents for pages that want it ----
    for page in &mut pages {
        let has_toc = page.metadata.get("toc").is_some_and(|v| v == "true");
        if has_toc {
            let toc = generate_toc(&page.content);
            if !toc.is_empty() {
                page.content.insert(0, Block::TableOfContents(toc));
            }
        }
    }

    // ---- Serialize with postcard ----
    let site_data = SiteData { pages };
    let bytes = postcard::to_allocvec(&site_data).expect("failed to serialize SiteData");

    // ---- Emit the bytes as a static array ----
    let byte_literals: Vec<proc_macro2::TokenStream> = bytes.iter().map(|b| quote! { #b }).collect();
    let len = bytes.len();

    // ---- Also emit rerun-if-changed directives ----
    let _content_path = content_dir.display().to_string();

    let output = quote! {
        {
            const _RERUN: &str = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/content/posts",
            );
            pub static SITE_BYTES: &[u8; #len] = &[#(#byte_literals),*];
            SITE_BYTES
        }
    };

    return output.into();
}

// ============================================================
// File reading
// ============================================================

fn content_dir() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    return PathBuf::from(manifest_dir).join("content");
}

fn read_posts(posts_dir: &PathBuf) -> Vec<Page> {
    let mut pages = Vec::new();

    if !posts_dir.exists() {
        return pages;
    }

    collect_markdown_files(posts_dir, posts_dir, &mut pages);
    pages.sort_by(|a, b| a.slug.cmp(&b.slug));
    return pages;
}

fn collect_markdown_files(base: &PathBuf, dir: &PathBuf, pages: &mut Vec<Page>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("failed to read dir {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(base, &path, pages);
        } else if path.extension().is_some_and(|ext| ext == "md")
            && let Some(page) = read_single_post(base, &path)
        {
            pages.push(page);
        }
    }
}

fn read_single_post(base: &PathBuf, path: &PathBuf) -> Option<Page> {
    let content = fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    // Parse YAML frontmatter
    let (metadata, body) = parse_frontmatter(&content);

    // Determine slug from path
    let relative = path.strip_prefix(base).unwrap().with_extension("");
    let slug = relative.to_str().unwrap().replace('\\', "/");

    // Determine folder
    let folder = relative
        .parent()
        .map(|p| p.to_str().unwrap_or(""))
        .unwrap_or("")
        .replace('\\', "/");

    // Check for draft
    if metadata.get("draft").is_some_and(|v| v == "true") {
        return None;
    }

    let mut meta = metadata;
    meta.insert("body".to_string(), body);

    return Some(Page {
        slug,
        folder,
        metadata: meta,
        content: Vec::new(),
        blocks: Vec::new(),
        citations: Vec::new(),
        internal_links: Vec::new(),
        external_links: Vec::new(),
        sources: Vec::new(),
    });
}

// ============================================================
// Table of contents generation
// ============================================================

fn generate_toc(content: &[Block]) -> Vec<TocEntry> {
    let mut toc = Vec::new();
    for block in content {
        if let Block::Heading {
            level,
            id,
            number,
            children,
        } = block
        {
            toc.push(TocEntry {
                level: *level,
                id: id.clone(),
                title: ir::parse::inlines_to_plain_text(children),
                number: number.clone(),
            });
        }
    }
    return toc;
}
