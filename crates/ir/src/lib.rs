#![allow(clippy::needless_return)]

pub mod bib;
pub mod frontmatter;
pub mod parse;
pub mod resolve;
pub mod types;

pub use bib::{BibEntry, parse_bib_file, parse_bib_str};
pub use types::*;

// ============================================================
// Shared utilities
// ============================================================

/// Capitalize the first character of a string.
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
