#![allow(clippy::needless_return)]

//! Core IR types for the content pipeline.
//!
//! Every markdown post is parsed into a `Page` containing a tree of `Block` and
//! `Inline` nodes.  Cross-references, citations, and auto-links are represented
//! as first-class nodes rather than ad-hoc string substitutions.
//!
//! The types derive `Serialize`/`Deserialize` so the proc-macro can embed the
//! entire site as a compact byte blob and the frontend can deserialize it once
//! at startup — zero token-stream gymnastics, zero `&'static` lifetime juggling.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ============================================================
// Inline nodes
// ============================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Math(String),
    Link {
        url: String,
        children: Vec<Inline>,
    },
    Image {
        url: String,
        alt: String,
        title: String,
    },
    CrossRef {
        label: String,
        /// Empty string means auto-generate display text from the target block.
        display: String,
        /// Block kind (e.g. "definition", "theorem"). Populated during resolution.
        kind: String,
        /// Block title. Populated during resolution.
        title: String,
        /// Block number. Populated during resolution.
        number: String,
        /// Plain-text preview for tooltips. Populated during resolution.
        content_preview: String,
    },
    Citation {
        key: String,
        /// Free-form locator, e.g. "Thm 2", "§3.1".  Empty if absent.
        locator: String,
        /// Unique anchor id for backlink (e.g. "cite-ref-1").
        anchor_id: String,
        /// Pre-resolved display text, e.g. "Lang, 2002".
        display: String,
    },
    FootnoteRef(String),
    SoftBreak,
    HardBreak,
    Html(String),
}

// ============================================================
// Block nodes
// ============================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Block {
    // ---- structure ----
    Heading {
        level: u8,
        id: String,
        /// Section number string (e.g. "1", "2.1"). Empty when TOC is disabled.
        number: String,
        children: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),

    // ---- code ----
    CodeBlock {
        language: String,
        code: String,
    },

    // ---- academic blocks ----
    LabeledBlock {
        kind: String,
        id: String,
        label: String,
        number: String,
        title: String,
        content: Vec<Block>,
    },
    Equation {
        id: String,
        label: String,
        number: String,
        title: String,
        /// Raw LaTeX source (no `$$` delimiters).
        latex: String,
    },
    Diagram {
        variant: DiagramKind,
        id: String,
        label: String,
        title: String,
        /// Raw diagram source (TikZ or Mermaid).
        source: String,
    },
    Figure {
        id: String,
        label: String,
        number: String,
        title: String,
        content: Vec<Block>,
    },

    // ---- math ----
    MathDisplay(String),

    // ---- standard blocks ----
    Callout {
        kind: String,
        content: Vec<Block>,
    },
    Blockquote(Vec<Block>),
    List {
        ordered: bool,
        start: u64,
        items: Vec<Vec<Block>>,
    },
    Table {
        alignments: Vec<Alignment>,
        header: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    FootnoteDef {
        id: String,
        content: Vec<Block>,
    },
    TableOfContents(Vec<TocEntry>),
    ThematicBreak,
    Html(String),
}

// ============================================================
// Supporting types
// ============================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DiagramKind {
    Tikz,
    TikzCd,
    Mermaid,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TocEntry {
    pub level: u8,
    pub id: String,
    pub title: String,
    /// Section number string, e.g. "1", "1.1", "2.3".
    pub number: String,
}

// ============================================================
// Block metadata (for hover previews, search, pinned panel)
// ============================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMeta {
    pub label: String,
    pub kind: String,
    pub title: String,
    pub aliases: Vec<String>,
    pub number: String,
    /// Plain-text or light-HTML preview for tooltips.
    pub content_preview: String,
}

// ============================================================
// Citation metadata
// ============================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CitationMeta {
    pub key: String,
    /// Display label for the citation (e.g. "Lan02" in AMS alpha style).
    pub label: String,
    /// Pre-rendered HTML for the bibliography entry.
    pub formatted_html: String,
    /// Anchor ids in the body that cite this entry (for backlinks).
    pub backlink_ids: Vec<String>,
}

// ============================================================
// Page (one markdown file)
// ============================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Page {
    pub slug: String,
    pub folder: String,
    pub metadata: HashMap<String, String>,
    pub content: Vec<Block>,

    // ---- derived data (populated during resolution) ----
    pub blocks: Vec<BlockMeta>,
    pub citations: Vec<CitationMeta>,
    pub internal_links: Vec<String>,
    pub external_links: Vec<String>,
    pub sources: Vec<String>,
}

// ============================================================
// Link tracking
// ============================================================

/// Maps a URL or anchor href to the list of inline anchor-ids where it appears
/// in the rendered body. Used to build backlinks (↑1, ↑2, …).
pub type LinkOccurrences = HashMap<String, Vec<String>>;

// ============================================================
// Site data (all pages + bibliography)
// ============================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteData {
    pub pages: Vec<Page>,
}
