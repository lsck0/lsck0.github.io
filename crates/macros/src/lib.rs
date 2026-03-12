#![allow(clippy::needless_return)]

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn include_posts(_input: TokenStream) -> TokenStream {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let content_dir = PathBuf::from(&manifest_dir).join("content");

    let mut posts: Vec<(String, String, BTreeMap<String, String>, String)> = Vec::new();

    if content_dir.exists() {
        walk_dir(&content_dir, &content_dir, &mut posts);
    }

    // Sort by date descending
    posts.sort_by(|a, b| {
        let date_a = a.2.get("date").map(|s| s.as_str()).unwrap_or("");
        let date_b = b.2.get("date").map(|s| s.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });

    let post_tokens = posts.iter().map(|(slug, folder, metadata, body)| {
        let meta_entries = metadata.iter().map(|(k, v)| {
            quote! { (#k, #v) }
        });

        quote! {
            Post {
                slug: #slug,
                folder: #folder,
                metadata: &[#(#meta_entries),*],
                content: #body,
            }
        }
    });

    let output = quote! {
        &[#(#post_tokens),*]
    };

    return output.into();
}

fn walk_dir(dir: &Path, base: &Path, posts: &mut Vec<(String, String, BTreeMap<String, String>, String)>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("failed to read content directory")
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, base, posts);
        } else if path.extension().is_some_and(|x| x == "md") {
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let folder = path
                .parent()
                .unwrap()
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .to_string();
            let slug = if folder.is_empty() {
                stem
            } else {
                format!("{}/{}", folder, path.file_stem().unwrap().to_string_lossy())
            };
            let content = fs::read_to_string(&path).expect("failed to read markdown file");
            let (metadata, body) = parse_frontmatter(&content);
            validate_frontmatter(&metadata, &path);
            posts.push((slug, folder, metadata, body));
        }
    }
}

const REQUIRED_FIELDS: &[&str] = &["title", "date"];
const OPTIONAL_FIELDS: &[&str] = &["tags", "description", "project", "publication"];

fn validate_frontmatter(metadata: &BTreeMap<String, String>, path: &Path) {
    let display = path.display();

    for field in REQUIRED_FIELDS {
        if !metadata.contains_key(*field) {
            panic!("{display}: missing required frontmatter field \"{field}\"");
        }
    }

    for key in metadata.keys() {
        if !REQUIRED_FIELDS.contains(&key.as_str()) && !OPTIONAL_FIELDS.contains(&key.as_str()) {
            panic!("{display}: unknown frontmatter field \"{key}\"");
        }
    }
}

fn parse_frontmatter(content: &str) -> (BTreeMap<String, String>, String) {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return (BTreeMap::new(), content.to_string());
    }

    let after_marker = &content[3..];
    let after_marker = after_marker
        .strip_prefix('\n')
        .or_else(|| after_marker.strip_prefix("\r\n"))
        .unwrap_or(after_marker);

    let end = after_marker.find("\n---").unwrap_or(after_marker.len());
    let fm = &after_marker[..end];
    let body_start = end + 4;
    let body = if body_start < after_marker.len() {
        let rest = &after_marker[body_start..];
        rest.strip_prefix('\n')
            .or_else(|| rest.strip_prefix("\r\n"))
            .unwrap_or(rest)
    } else {
        ""
    };

    let mut metadata = BTreeMap::new();
    for line in fm.lines() {
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim().to_string();
            let val = val.trim();
            let val = val
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(val)
                .to_string();
            if !key.is_empty() {
                metadata.insert(key, val);
            }
        }
    }

    return (metadata, body.to_string());
}
