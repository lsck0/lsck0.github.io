#![allow(clippy::needless_return)]

// Re-export the shared content types and functions.
// The indexer uses ParsedPost directly from the content crate.
pub use content::{ParsedPost as ContentPost, parse_posts_directory, resolve_transclusions};
