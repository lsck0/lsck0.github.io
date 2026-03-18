#![allow(clippy::needless_return)]

mod feeds;
mod index;
mod parse;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use feeds::{
    build_atom_feed, build_jsonld, build_opengraph_metadata, build_rss_feed, build_sitemap, inject_opengraph_tags,
};
use index::{build_graph_data, build_search_index};
use parse::{parse_posts_directory, resolve_transclusions};

// ============================================================
// Configuration (loaded from content/meta.toml)
// ============================================================

struct SiteConfig {
    url: String,
    title: String,
    description: String,
    author: String,
}

fn load_site_config(project_root: &Path) -> SiteConfig {
    let meta_path = project_root.join("content/meta.toml");
    let meta_str = fs::read_to_string(&meta_path).unwrap_or_else(|_| panic!("failed to read: {}", meta_path.display()));
    let meta: toml::Value = meta_str.parse().expect("failed to parse meta.toml");

    let site = meta.get("site").expect("meta.toml missing [site] section");
    let get = |key| {
        site.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("meta.toml missing site.{key}"))
            .to_string()
    };

    return SiteConfig {
        url: get("url"),
        title: get("title"),
        description: get("description"),
        author: get("author"),
    };
}

// ============================================================
// Main
// ============================================================

fn main() {
    let project_root = env::var("CARGO_MANIFEST_DIR")
        .map(|dir| PathBuf::from(dir).join("../.."))
        .unwrap_or_else(|_| PathBuf::from("."));

    let config = load_site_config(&project_root);

    let content_dir = project_root.join("content/posts");
    let output_dir = project_root.join("dist");

    println!("Parsing posts from: {}", content_dir.display());

    let mut posts = parse_posts_directory(&content_dir);
    resolve_transclusions(&mut posts);

    let published: Vec<&parse::ContentPost> = posts.iter().filter(|p| !p.is_draft()).collect();

    // ---- Inject site metadata into index.html ----
    let index_html_path = output_dir.join("index.html");
    if index_html_path.exists() {
        let html = fs::read_to_string(&index_html_path).expect("failed to read index.html");
        let patched = inject_site_meta(&html, &config);
        fs::write(&index_html_path, &patched).expect("failed to write index.html");
        println!("  patched: index.html with meta.toml values");
    }

    // ---- JSON indexes ----
    let graph_data = build_graph_data(&published);
    let search_index = build_search_index(&published);

    write_file(
        &output_dir.join("graph.json"),
        &serde_json::to_string_pretty(&graph_data).expect("failed to serialize graph"),
    );
    write_file(
        &output_dir.join("search_index.json"),
        &serde_json::to_string_pretty(&search_index).expect("failed to serialize search index"),
    );

    // ---- Feeds ----
    write_file(
        &output_dir.join("rss.xml"),
        &build_rss_feed(&published, &config.url, &config.title),
    );
    write_file(
        &output_dir.join("atom.xml"),
        &build_atom_feed(&published, &config.url, &config.title),
    );
    write_file(&output_dir.join("sitemap.xml"), &build_sitemap(&published, &config.url));

    // ---- llms.txt ----
    write_file(&output_dir.join("llms.txt"), &build_llms_txt());

    // ---- OG meta tag pages + JSON-LD ----
    if index_html_path.exists() {
        let base_html = fs::read_to_string(&index_html_path).expect("failed to read index.html");
        let og_entries = build_opengraph_metadata(&published, &config.url);

        for entry in &og_entries {
            let mut post_html = inject_opengraph_tags(&base_html, entry, &config.title);
            let jsonld = build_jsonld(entry, &config.url);
            post_html = post_html.replace("</head>", &format!("{jsonld}\n  </head>"));

            let post_dir = output_dir.join("blog").join(&entry.slug);
            write_file(&post_dir.join("index.html"), &post_html);
        }
        println!("Generated OG pages for {} posts.", og_entries.len());
    } else {
        println!("Warning: index.html not found, skipping OG page generation.");
    }

    println!("Indexer complete. Output: {}", output_dir.display());
}

// ============================================================
// Inject site metadata into index.html (single source of truth: meta.toml)
// ============================================================

fn inject_site_meta(html: &str, config: &SiteConfig) -> String {
    let mut result = html.to_string();

    // Replace placeholder meta tags with values from meta.toml
    // These use data-meta="..." attributes as stable anchors for replacement.
    result = result.replace(
        "content=\"__META_DESCRIPTION__\"",
        &format!("content=\"{}\"", config.description),
    );
    result = result.replace("content=\"__META_AUTHOR__\"", &format!("content=\"{}\"", config.author));
    result = result.replace(
        "content=\"__META_OG_TITLE__\"",
        &format!("content=\"{}\"", config.title),
    );
    result = result.replace(
        "content=\"__META_OG_DESCRIPTION__\"",
        &format!("content=\"{}\"", config.description),
    );
    result = result.replace("content=\"__META_OG_URL__\"", &format!("content=\"{}/\"", config.url));

    return result;
}

// ============================================================
// llms.txt
// ============================================================

fn build_llms_txt() -> String {
    return r#"# /dev/lsck0

> This website and all its content are copyrighted by Luca Sandrock.

## AI Training Opt-Out

This website explicitly opts out of AI model training.

All content on this site — including but not limited to blog posts, articles,
code samples, diagrams, and images — is NOT licensed for use in training
artificial intelligence or machine learning models.

You may NOT:
- Scrape this website for AI/ML training data
- Include any content from this site in training datasets
- Use content from this site to fine-tune language models
- Reproduce content from this site in AI-generated outputs without attribution

You MAY:
- Cite and reference this content with proper attribution
- Link to pages on this site
- Quote brief excerpts under fair use with attribution

For questions, contact the site owner.

## Legal

Violation of these terms may result in legal action under applicable
copyright and intellectual property laws.
"#
    .to_string();
}

// ============================================================
// File writing helper
// ============================================================

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|_| panic!("failed to create dir: {}", parent.display()));
    }
    fs::write(path, content).unwrap_or_else(|_| panic!("failed to write: {}", path.display()));
    println!("  wrote: {}", path.display());
}
