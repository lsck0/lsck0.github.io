#![allow(clippy::needless_return)]

//! Proc macro that reads `content/macros.tex` and compiles it into a JSON
//! string suitable for KaTeX's `macros` option.
//!
//! Supports `\newcommand{\name}{body}` and `\newcommand{\name}[N]{body}`.

use std::{env, path::PathBuf};

use proc_macro::TokenStream;
use quote::quote;

pub fn include_macros_impl(_input: TokenStream) -> TokenStream {
    let content_dir = content_dir();
    let macros_path = content_dir.join("macros.tex");

    let json = if macros_path.exists() {
        let content =
            std::fs::read_to_string(&macros_path).unwrap_or_else(|e| panic!("failed to read macros.tex: {e}"));
        parse_macros_to_json(&content)
    } else {
        "{}".to_string()
    };

    let output = quote! { #json };
    return output.into();
}

fn content_dir() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    return PathBuf::from(manifest_dir).join("content");
}

/// Parse a `.tex` file with `\newcommand` / `\renewcommand` lines into a JSON object string.
fn parse_macros_to_json(input: &str) -> String {
    let mut entries: Vec<(String, String)> = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') {
            continue;
        }

        // Match \newcommand or \renewcommand
        let rest = if let Some(r) = trimmed.strip_prefix("\\newcommand") {
            r
        } else if let Some(r) = trimmed.strip_prefix("\\renewcommand") {
            r
        } else {
            continue;
        };

        if let Some((name, body)) = parse_command(rest) {
            entries.push((name, body));
        }
    }

    // Build JSON manually to avoid pulling in serde_json in proc macro
    let mut json = String::from("{");
    for (i, (name, body)) in entries.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str(&json_escape(name));
        json.push_str("\":\"");
        json.push_str(&json_escape(body));
        json.push('"');
    }
    json.push('}');
    return json;
}

/// Parse `{\name}[N]{body}` or `{\name}{body}` from the remainder after `\newcommand`.
fn parse_command(input: &str) -> Option<(String, String)> {
    let input = input.trim();

    // Extract command name from {\name}
    let input = input.strip_prefix('{')?;
    let close = input.find('}')?;
    let name = input[..close].to_string();
    let mut rest = input[close + 1..].trim();

    // Optional argument count [N] — we don't need the count, KaTeX infers it from #1, #2, etc.
    if rest.starts_with('[')
        && let Some(bracket_close) = rest.find(']')
    {
        rest = rest[bracket_close + 1..].trim();
    }

    // Extract body from {body}
    let body = extract_braced(rest)?;

    return Some((name, body));
}

/// Extract the content of a brace-delimited group, handling nested braces.
fn extract_braced(input: &str) -> Option<String> {
    let input = input.trim();
    if !input.starts_with('{') {
        return None;
    }
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in input.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    start = i + 1;
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(input[start..i].trim().to_string());
                }
            }
            _ => {}
        }
    }
    return None;
}

fn json_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(ch),
        }
    }
    return result;
}
