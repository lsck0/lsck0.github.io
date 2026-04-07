#![allow(clippy::needless_return)]

//! Markdown → IR parser.
//!
//! Uses pulldown-cmark for standard markdown, then handles our custom extensions:
//! - Labeled blocks:  ```definition/theorem/equation/tikzcd/mermaid/...
//! - Cross-references: [[label]]
//! - Citations:        [@key] and [@key, locator]
//! - Transclusion:     ![[slug]]
//! - Callouts:         ```tip/warning/... or > [!tip]

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::types::*;

// ============================================================
// Block kind classification
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
    "figure",
    "equation",
];

const UNNUMBERED_KINDS: &[&str] = &["proof", "tikz", "tikzcd", "mermaid"];

const CALLOUT_KINDS: &[&str] = &["tip", "warning", "danger", "note", "info"];

fn is_academic_kind(kind: &str) -> bool {
    return NUMBERED_KINDS.contains(&kind) || UNNUMBERED_KINDS.contains(&kind);
}

fn is_callout_kind(kind: &str) -> bool {
    return CALLOUT_KINDS.contains(&kind);
}

fn is_numbered_kind(kind: &str) -> bool {
    return NUMBERED_KINDS.contains(&kind);
}

// ============================================================
// Numbering state
// ============================================================

struct NumberingState {
    /// Global counter for definitions/theorems etc (when no ToC sectioning).
    global_counter: u32,
    /// Section-scoped counter (resets at each heading).
    section_counter: u32,
    /// Section number parts (up to 3 levels, offset by min_level).
    section: [u32; 3],
    /// Separate counter for equations.
    equation_counter: u32,
    /// Separate counter for figures.
    figure_counter: u32,
    /// Whether the document uses table-of-contents (section-scoped numbering).
    has_toc: bool,
    /// Minimum heading level seen (e.g. 2 if post starts with ##).
    /// Used to normalize section indices so ## acts like top-level.
    min_level: u8,
}

impl NumberingState {
    fn new(has_toc: bool) -> Self {
        return Self {
            global_counter: 0,
            section_counter: 0,
            section: [0, 0, 0],
            equation_counter: 0,
            figure_counter: 0,
            has_toc,
            min_level: 0,
        };
    }

    fn on_heading(&mut self, level: u8) {
        if !self.has_toc {
            return;
        }
        if self.min_level == 0 {
            self.min_level = level;
        }
        let adjusted = (level.saturating_sub(self.min_level)) as usize;
        if adjusted < 3 {
            self.section[adjusted] += 1;
            for s in &mut self.section[adjusted + 1..] {
                *s = 0;
            }
            self.section_counter = 0;
        }
    }

    /// Current section number string for the most recent heading (e.g. "1", "2.1").
    fn section_number(&self) -> String {
        if !self.has_toc || self.section[0] == 0 {
            return String::new();
        }
        let depth = if self.section[2] > 0 {
            3
        } else if self.section[1] > 0 {
            2
        } else {
            1
        };
        return self.section[..depth]
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(".");
    }

    fn next_number(&mut self, kind: &str) -> String {
        if kind == "equation" {
            self.equation_counter += 1;
            return self.equation_counter.to_string();
        }
        if kind == "figure" {
            self.figure_counter += 1;
            return self.figure_counter.to_string();
        }
        if !is_numbered_kind(kind) {
            return String::new();
        }
        if self.has_toc && self.section[0] > 0 {
            self.section_counter += 1;
            let section_prefix = self.section_number();
            return format!("{section_prefix}.{}", self.section_counter);
        }
        self.global_counter += 1;
        return self.global_counter.to_string();
    }
}

// ============================================================
// Block header parsing
// ============================================================

/// Parse `kind Title | alias1, alias2 {#label}` from a code fence info string.
/// Returns (kind, title, aliases, optional_label).
fn parse_block_header(header: &str) -> Option<(String, String, Vec<String>, Option<String>)> {
    let kind_end = header.find(|c: char| c.is_whitespace()).unwrap_or(header.len());
    let kind = header[..kind_end].to_lowercase();
    let after_kind = header[kind_end..].trim();

    let (title_and_aliases, label) = if let Some(label_start) = after_kind.rfind("{#") {
        let after = &after_kind[label_start + 2..];
        if let Some(label_end) = after.find('}') {
            (
                after_kind[..label_start].trim().to_string(),
                Some(after[..label_end].to_string()),
            )
        } else {
            panic!("Unclosed label brace in block header: \"{header}\".");
        }
    } else {
        (after_kind.to_string(), None)
    };

    let (title, aliases) = if let Some(pipe_pos) = title_and_aliases.find('|') {
        let title = title_and_aliases[..pipe_pos].trim().to_string();
        let aliases: Vec<String> = title_and_aliases[pipe_pos + 1..]
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
// Top-level parse entry point
// ============================================================

/// Parse a markdown document body into IR blocks.
/// This handles our custom extensions on top of pulldown-cmark.
///
/// Returns (blocks, block_metadata) where block_metadata contains info
/// about every labeled block for the cross-reference registry.
pub fn parse_markdown(body: &str, has_toc: bool) -> (Vec<Block>, Vec<BlockMeta>) {
    let mut numbering = NumberingState::new(has_toc);
    let mut blocks = Vec::new();
    let mut block_metas = Vec::new();

    // Phase 1: Extract custom fenced blocks and callouts from the raw markdown,
    //          replacing them with placeholder HTML comments.
    //          Everything else is left as standard markdown for pulldown-cmark.
    let (processed_body, extracted, heading_numbers, nested_metas) = extract_custom_blocks(body, &mut numbering);

    // Phase 1b: Replace [[cross-refs]] and [@citations] with HTML comment
    //           placeholders so pulldown-cmark doesn't interpret the brackets
    //           as link syntax.
    let processed_body = escape_custom_inline_syntax(&processed_body);

    // Phase 2: Parse the remaining markdown with pulldown-cmark.
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_MATH | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(&processed_body, options);
    let events: Vec<Event> = parser.collect();

    // Phase 3: Convert pulldown-cmark events to IR blocks, re-inserting
    //          extracted custom blocks at their placeholder positions.
    let mut ctx = ConvertContext::new(&extracted, heading_numbers);
    convert_events(&events, &mut blocks, &mut ctx);

    // Collect block metadata
    for ext in &extracted {
        block_metas.push(ext.to_meta());
    }
    // Include metas from nested parse_markdown calls (e.g. equation labels inside definition blocks)
    block_metas.extend(nested_metas);

    return (blocks, block_metas);
}

// ============================================================
// Phase 1b: Escape custom inline syntax before pulldown-cmark
// ============================================================

/// Replace `[[label]]`, `[[label|display]]`, and `[@key]` / `[@key, loc]` / `[@k1; @k2]`
/// with HTML comment placeholders so pulldown-cmark doesn't interpret the brackets
/// as broken link references.
fn escape_custom_inline_syntax(body: &str) -> String {
    let mut result = String::with_capacity(body.len());
    let mut chars = body.char_indices().peekable();

    while let Some(&(pos, ch)) = chars.peek() {
        // Transclusion ![[slug]]
        if ch == '!'
            && body[pos..].starts_with("![[")
            && let Some(end) = body[pos + 3..].find("]]")
        {
            let inner = &body[pos + 3..pos + 3 + end];
            result.push_str(&format!("<!--TRCL:{inner}-->"));
            for _ in 0..3 + end + 2 {
                chars.next();
            }
            continue;
        }

        // Cross-reference [[label]] or [[label|display]]
        if ch == '['
            && body[pos..].starts_with("[[")
            && !body[pos..].starts_with("[[@")
            && let Some(end) = body[pos + 2..].find("]]")
        {
            let inner = &body[pos + 2..pos + 2 + end];
            if !inner.contains('\n') && !inner.is_empty() {
                result.push_str(&format!("<!--XREF:{inner}-->"));
                for _ in 0..2 + end + 2 {
                    chars.next();
                }
                continue;
            }
        }

        // Citation [@key] or [@key, locator] or [@key1; @key2]
        if ch == '['
            && body[pos..].starts_with("[@")
            && let Some(end) = body[pos + 1..].find(']')
        {
            let inner = &body[pos + 1..pos + 1 + end];
            if !inner.contains('\n') {
                result.push_str(&format!("<!--CITE:{inner}-->"));
                for _ in 0..1 + end + 1 {
                    chars.next();
                }
                continue;
            }
        }

        result.push(ch);
        chars.next();
    }

    return result;
}

// ============================================================
// Phase 1: Extract custom fenced blocks
// ============================================================

struct ExtractedBlock {
    placeholder_id: String,
    block: Block,
    label: String,
    kind: String,
    title: String,
    aliases: Vec<String>,
    number: String,
    content_preview: String,
}

impl ExtractedBlock {
    fn to_meta(&self) -> BlockMeta {
        return BlockMeta {
            label: self.label.clone(),
            kind: self.kind.clone(),
            title: self.title.clone(),
            aliases: self.aliases.clone(),
            number: self.number.clone(),
            content_preview: self.content_preview.clone(),
        };
    }
}

fn extract_custom_blocks(
    body: &str,
    numbering: &mut NumberingState,
) -> (String, Vec<ExtractedBlock>, Vec<String>, Vec<BlockMeta>) {
    let mut result = String::new();
    let mut extracted = Vec::new();
    let mut heading_numbers = Vec::new();
    let mut inner_metas = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Check for backtick-fenced block FIRST (to avoid counting headings inside code blocks)
        let backtick_count = trimmed.chars().take_while(|&c| c == '`').count();
        if backtick_count >= 3 {
            let info = trimmed[backtick_count..].trim();
            let fence_len = backtick_count;
            let opening_line = lines[i];

            // check if this is a custom block (academic or callout)
            let custom_block = if !info.is_empty() {
                parse_block_header(info).and_then(|(kind, title, aliases, label_opt)| {
                    let kind_str = kind.as_str();
                    if is_academic_kind(kind_str) || is_callout_kind(kind_str) {
                        Some((kind, title, aliases, label_opt))
                    } else {
                        None
                    }
                })
            } else {
                None
            };

            // collect content until closing fence (for all fenced blocks)
            i += 1;
            let mut content = String::new();
            let mut closing_line = "";
            while i < lines.len() {
                let line_trimmed = lines[i].trim();
                let closing_backticks = line_trimmed.chars().take_while(|&c| c == '`').count();
                if closing_backticks >= fence_len && line_trimmed[closing_backticks..].trim().is_empty() {
                    closing_line = lines[i];
                    break;
                }
                if !content.is_empty() {
                    content.push('\n');
                }
                content.push_str(lines[i]);
                i += 1;
            }
            i += 1; // skip closing fence

            if let Some((kind, title, aliases, label_opt)) = custom_block {
                // custom block - extract and replace with placeholder
                let kind_str = kind.as_str();
                let content = content.trim().to_string();

                if is_callout_kind(kind_str) {
                    // Callout — parse content as markdown
                    let (inner_blocks, nested_metas) = parse_markdown(&content, false);
                    inner_metas.extend(nested_metas);
                    let placeholder = format!("<!--PLACEHOLDER-{}-->", extracted.len());
                    extracted.push(ExtractedBlock {
                        placeholder_id: placeholder.clone(),
                        block: Block::Callout {
                            kind: kind.clone(),
                            content: inner_blocks,
                        },
                        label: String::new(),
                        kind: kind.clone(),
                        title: String::new(),
                        aliases: Vec::new(),
                        number: String::new(),
                        content_preview: content.clone(),
                    });
                    result.push_str(&placeholder);
                    result.push('\n');
                } else {
                    // Academic block
                    let number = numbering.next_number(kind_str);
                    let label = label_opt.unwrap_or_else(|| {
                        if !number.is_empty() {
                            format!("{kind}-{number}")
                        } else {
                            format!("{kind}-{}", extracted.len())
                        }
                    });
                    let id = label.replace(':', "-");

                    let block = match kind_str {
                        "equation" => Block::Equation {
                            id: id.clone(),
                            label: label.clone(),
                            number: number.clone(),
                            title: title.clone(),
                            latex: content.clone(),
                        },
                        "tikz" => Block::Diagram {
                            variant: DiagramKind::Tikz,
                            id: id.clone(),
                            label: label.clone(),
                            title: title.clone(),
                            source: content.clone(),
                        },
                        "tikzcd" => Block::Diagram {
                            variant: DiagramKind::TikzCd,
                            id: id.clone(),
                            label: label.clone(),
                            title: title.clone(),
                            source: content.clone(),
                        },
                        "mermaid" => Block::Diagram {
                            variant: DiagramKind::Mermaid,
                            id: id.clone(),
                            label: label.clone(),
                            title: title.clone(),
                            source: content.clone(),
                        },
                        "figure" => {
                            let (inner_blocks, nested_metas) = parse_markdown(&content, false);
                            inner_metas.extend(nested_metas);
                            Block::Figure {
                                id: id.clone(),
                                label: label.clone(),
                                number: number.clone(),
                                title: title.clone(),
                                content: inner_blocks,
                            }
                        }
                        _ => {
                            // definition, theorem, proof, etc. — parse content as markdown
                            let (inner_blocks, nested_metas) = parse_markdown(&content, false);
                            inner_metas.extend(nested_metas);
                            Block::LabeledBlock {
                                kind: kind.clone(),
                                id: id.clone(),
                                label: label.clone(),
                                number: number.clone(),
                                title: title.clone(),
                                content: inner_blocks,
                            }
                        }
                    };

                    let placeholder = format!("<!--PLACEHOLDER-{}-->", extracted.len());
                    extracted.push(ExtractedBlock {
                        placeholder_id: placeholder.clone(),
                        block,
                        label: label.clone(),
                        kind: kind.clone(),
                        title: title.clone(),
                        aliases,
                        number: number.clone(),
                        content_preview: content,
                    });
                    result.push_str(&placeholder);
                    result.push('\n');
                }
            } else {
                // regular code block - pass through unchanged (headings inside are skipped)
                result.push_str(opening_line);
                result.push('\n');
                if !content.is_empty() {
                    result.push_str(&content);
                    result.push('\n');
                }
                result.push_str(closing_line);
                result.push('\n');
            }
            continue;
        }

        // Track headings for section-scoped numbering (after skipping code blocks)
        if trimmed.starts_with('#')
            && trimmed.chars().nth(trimmed.chars().take_while(|&c| c == '#').count()) == Some(' ')
        {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            numbering.on_heading(level as u8);
            heading_numbers.push(if numbering.has_toc {
                numbering.section_number()
            } else {
                String::new()
            });
        }

        // Check for blockquote callout syntax: > [!tip]
        if trimmed.starts_with("> [!")
            && let Some(kind_end) = trimmed[4..].find(']')
        {
            let kind = trimmed[4..4 + kind_end].to_lowercase();
            if is_callout_kind(&kind) {
                // Collect all lines that are part of this blockquote
                let mut bq_content = String::new();
                i += 1;
                while i < lines.len() {
                    let line = lines[i];
                    if let Some(stripped) = line.strip_prefix("> ") {
                        bq_content.push_str(stripped);
                        bq_content.push('\n');
                    } else if line.trim() == ">" {
                        bq_content.push('\n');
                    } else {
                        break;
                    }
                    i += 1;
                }

                let (inner_blocks, nested_metas) = parse_markdown(bq_content.trim(), false);
                inner_metas.extend(nested_metas);
                let placeholder = format!("<!--PLACEHOLDER-{}-->", extracted.len());
                extracted.push(ExtractedBlock {
                    placeholder_id: placeholder.clone(),
                    block: Block::Callout {
                        kind: kind.clone(),
                        content: inner_blocks,
                    },
                    label: String::new(),
                    kind,
                    title: String::new(),
                    aliases: Vec::new(),
                    number: String::new(),
                    content_preview: bq_content,
                });
                result.push_str(&placeholder);
                result.push('\n');
                continue;
            }
        }

        // Check for display math with equation label: $$...$$\n{#eq:label} or $$...$$ {#eq:label}
        if trimmed.starts_with("$$") {
            // Single-line display math: $$...$$
            if trimmed.ends_with("$$") && trimmed.len() > 4 {
                let latex = trimmed[2..trimmed.len() - 2].trim().to_string();
                // Check next line for {#eq:label}
                if let Some(label) = peek_eq_label(&lines, i + 1) {
                    let number = numbering.next_number("equation");
                    let id = label.replace(':', "-");
                    let placeholder = format!("<!--PLACEHOLDER-{}-->", extracted.len());
                    extracted.push(ExtractedBlock {
                        placeholder_id: placeholder.clone(),
                        block: Block::Equation {
                            id: id.clone(),
                            label: label.clone(),
                            number: number.clone(),
                            title: String::new(),
                            latex: latex.clone(),
                        },
                        label: label.clone(),
                        kind: "equation".to_string(),
                        title: String::new(),
                        aliases: Vec::new(),
                        number,
                        content_preview: latex,
                    });
                    result.push_str(&placeholder);
                    result.push('\n');
                    i += 2; // skip math line + label line
                    continue;
                }
            } else if trimmed == "$$" {
                // Multi-line display math: $$\n...\n$$
                let math_start = i + 1;
                let mut math_end = math_start;
                while math_end < lines.len() && lines[math_end].trim() != "$$" {
                    math_end += 1;
                }
                if math_end < lines.len() {
                    // Check line after closing $$ for {#eq:label}
                    if let Some(label) = peek_eq_label(&lines, math_end + 1) {
                        let latex = lines[math_start..math_end].join("\n").trim().to_string();
                        let number = numbering.next_number("equation");
                        let id = label.replace(':', "-");
                        let placeholder = format!("<!--PLACEHOLDER-{}-->", extracted.len());
                        extracted.push(ExtractedBlock {
                            placeholder_id: placeholder.clone(),
                            block: Block::Equation {
                                id: id.clone(),
                                label: label.clone(),
                                number: number.clone(),
                                title: String::new(),
                                latex: latex.clone(),
                            },
                            label: label.clone(),
                            kind: "equation".to_string(),
                            title: String::new(),
                            aliases: Vec::new(),
                            number,
                            content_preview: latex,
                        });
                        result.push_str(&placeholder);
                        result.push('\n');
                        i = math_end + 2; // skip past closing $$ + label line
                        continue;
                    }
                }
            }
            // Also check for inline label on same line: $$...$$ {#eq:label}
            if trimmed.contains("$$")
                && let Some(label_start) = trimmed.rfind("{#eq:")
                && let Some(label_end) = trimmed[label_start + 2..].find('}')
            {
                let label = trimmed[label_start + 2..label_start + 2 + label_end].to_string();
                let math_part = trimmed[..label_start].trim();
                if math_part.starts_with("$$") && math_part.ends_with("$$") && math_part.len() > 4 {
                    let latex = math_part[2..math_part.len() - 2].trim().to_string();
                    let number = numbering.next_number("equation");
                    let id = label.replace(':', "-");
                    let placeholder = format!("<!--PLACEHOLDER-{}-->", extracted.len());
                    extracted.push(ExtractedBlock {
                        placeholder_id: placeholder.clone(),
                        block: Block::Equation {
                            id: id.clone(),
                            label: label.clone(),
                            number: number.clone(),
                            title: String::new(),
                            latex: latex.clone(),
                        },
                        label: label.clone(),
                        kind: "equation".to_string(),
                        title: String::new(),
                        aliases: Vec::new(),
                        number,
                        content_preview: latex,
                    });
                    result.push_str(&placeholder);
                    result.push('\n');
                    i += 1;
                    continue;
                }
            }
        }

        result.push_str(lines[i]);
        result.push('\n');
        i += 1;
    }

    return (result, extracted, heading_numbers, inner_metas);
}

/// Check if the line at `idx` is an equation label like `{#eq:some_label}`.
fn peek_eq_label(lines: &[&str], idx: usize) -> Option<String> {
    if idx >= lines.len() {
        return None;
    }
    let trimmed = lines[idx].trim();
    if trimmed.starts_with("{#eq:") && trimmed.ends_with('}') {
        let label = trimmed[2..trimmed.len() - 1].to_string();
        return Some(label);
    }
    return None;
}

// ============================================================
// Phase 3: Convert pulldown-cmark events to IR
// ============================================================

struct ConvertContext<'a> {
    extracted: &'a [ExtractedBlock],
    /// Pre-computed heading numbers from Phase 1 (in document order).
    heading_numbers: Vec<String>,
    heading_index: usize,
}

impl<'a> ConvertContext<'a> {
    fn new(extracted: &'a [ExtractedBlock], heading_numbers: Vec<String>) -> Self {
        return Self {
            extracted,
            heading_numbers,
            heading_index: 0,
        };
    }

    fn next_heading_number(&mut self) -> String {
        let num = self
            .heading_numbers
            .get(self.heading_index)
            .cloned()
            .unwrap_or_default();
        self.heading_index += 1;
        return num;
    }

    fn find_placeholder(&self, text: &str) -> Option<&'a ExtractedBlock> {
        let trimmed = text.trim();
        self.extracted
            .iter()
            .find(|&ext| trimmed.contains(&ext.placeholder_id))
            .map(|v| v as _)
    }
}

fn convert_events(events: &[Event], blocks: &mut Vec<Block>, ctx: &mut ConvertContext) {
    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::Heading { level, .. }) => {
                i += 1;
                let children = collect_inlines(events, &mut i);
                let id = heading_id_from_inlines(&children);
                let number = ctx.next_heading_number();
                blocks.push(Block::Heading {
                    level: heading_level_to_u8(*level),
                    id,
                    number,
                    children,
                });
            }
            Event::Start(Tag::Paragraph) => {
                i += 1;

                // Check for display math paragraph: Start(Paragraph) → DisplayMath → End(Paragraph)
                // pulldown-cmark wraps `$$...$$` in a paragraph even when it's block-level.
                if let Some(Event::DisplayMath(math)) = events.get(i) {
                    let latex = math.to_string();
                    i += 1; // skip DisplayMath
                    // Skip End(Paragraph)
                    if i < events.len() && matches!(events[i], Event::End(TagEnd::Paragraph)) {
                        i += 1;
                    }
                    blocks.push(Block::MathDisplay(latex));
                    continue;
                }

                let children = collect_inlines(events, &mut i);

                // Check if this paragraph is a placeholder for an extracted block
                if children.len() == 1 {
                    if let Inline::Html(html) = &children[0]
                        && let Some(ext) = ctx.find_placeholder(html)
                    {
                        blocks.push(ext.block.clone());
                        i += 1;
                        continue;
                    }
                    if let Inline::Text(text) = &children[0]
                        && let Some(ext) = ctx.find_placeholder(text)
                    {
                        blocks.push(ext.block.clone());
                        i += 1;
                        continue;
                    }
                }
                // Check for placeholder within multiple inlines
                let mut found_placeholder = false;
                for inline in &children {
                    match inline {
                        Inline::Html(html) | Inline::Text(html) => {
                            if let Some(ext) = ctx.find_placeholder(html) {
                                blocks.push(ext.block.clone());
                                found_placeholder = true;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                if !found_placeholder {
                    blocks.push(Block::Paragraph(children));
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                let language = info.to_string();
                i += 1;
                let code = collect_raw_text(events, &mut i);

                // Handle bare tikz/tikzcd/mermaid code blocks (without labels)
                match language.split_whitespace().next().unwrap_or("") {
                    "tikz" => {
                        blocks.push(Block::Diagram {
                            variant: DiagramKind::Tikz,
                            id: String::new(),
                            label: String::new(),
                            title: String::new(),
                            source: code,
                        });
                    }
                    "tikzcd" => {
                        blocks.push(Block::Diagram {
                            variant: DiagramKind::TikzCd,
                            id: String::new(),
                            label: String::new(),
                            title: String::new(),
                            source: code,
                        });
                    }
                    "mermaid" => {
                        blocks.push(Block::Diagram {
                            variant: DiagramKind::Mermaid,
                            id: String::new(),
                            label: String::new(),
                            title: String::new(),
                            source: code,
                        });
                    }
                    _ => {
                        blocks.push(Block::CodeBlock { language, code });
                    }
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                i += 1;
                let code = collect_raw_text(events, &mut i);
                blocks.push(Block::CodeBlock {
                    language: String::new(),
                    code,
                });
            }
            Event::Start(Tag::BlockQuote(_)) => {
                i += 1;
                let mut inner = Vec::new();
                while i < events.len() {
                    if matches!(events[i], Event::End(TagEnd::BlockQuote(_))) {
                        break;
                    }
                    convert_events(&events[i..i + 1], &mut inner, ctx);
                    i += 1;
                }
                blocks.push(Block::Blockquote(inner));
            }
            Event::Start(Tag::List(start)) => {
                let ordered = start.is_some();
                let start_num = start.unwrap_or(1);
                i += 1;
                let mut items = Vec::new();
                while i < events.len() {
                    if matches!(events[i], Event::End(TagEnd::List(_))) {
                        break;
                    }
                    if matches!(events[i], Event::Start(Tag::Item)) {
                        i += 1;
                        // Collect all events inside this list item
                        let item_start = i;
                        let mut depth = 1;
                        while i < events.len() {
                            match &events[i] {
                                Event::Start(Tag::Item) => depth += 1,
                                Event::End(TagEnd::Item) => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                                _ => {}
                            }
                            i += 1;
                        }
                        let item_events = &events[item_start..i];
                        let mut item_blocks = Vec::new();

                        // For tight lists, pulldown-cmark emits inline events
                        // directly (no Paragraph wrapper). Detect this and wrap.
                        let has_block_start = item_events
                            .iter()
                            .any(|e| matches!(e, Event::Start(Tag::Paragraph | Tag::List(_) | Tag::BlockQuote(_))));

                        if has_block_start || item_events.is_empty() {
                            convert_events(item_events, &mut item_blocks, ctx);
                        } else {
                            // Tight list item: wrap inline events in a synthetic paragraph
                            let mut synth = Vec::with_capacity(item_events.len() + 2);
                            synth.push(Event::Start(Tag::Paragraph));
                            synth.extend_from_slice(item_events);
                            synth.push(Event::End(TagEnd::Paragraph));
                            convert_events(&synth, &mut item_blocks, ctx);
                        }

                        items.push(item_blocks);
                    }
                    i += 1;
                }
                blocks.push(Block::List {
                    ordered,
                    start: start_num,
                    items,
                });
            }
            Event::Start(Tag::Table(alignments)) => {
                let aligns: Vec<Alignment> = alignments
                    .iter()
                    .map(|a| match a {
                        pulldown_cmark::Alignment::None => Alignment::None,
                        pulldown_cmark::Alignment::Left => Alignment::Left,
                        pulldown_cmark::Alignment::Center => Alignment::Center,
                        pulldown_cmark::Alignment::Right => Alignment::Right,
                    })
                    .collect();
                i += 1;
                let mut header = Vec::new();
                let mut rows = Vec::new();
                let mut in_head = false;
                let mut current_row: Vec<Vec<Inline>> = Vec::new();

                while i < events.len() {
                    match &events[i] {
                        Event::End(TagEnd::Table) => break,
                        Event::Start(Tag::TableHead) => {
                            in_head = true;
                            current_row = Vec::new();
                        }
                        Event::End(TagEnd::TableHead) => {
                            header = current_row.clone();
                            in_head = false;
                        }
                        Event::Start(Tag::TableRow) => {
                            current_row = Vec::new();
                        }
                        Event::End(TagEnd::TableRow) if !in_head => {
                            rows.push(current_row.clone());
                        }
                        Event::Start(Tag::TableCell) => {
                            i += 1;
                            let cell = collect_inlines(events, &mut i);
                            current_row.push(cell);
                            i += 1;
                            continue;
                        }
                        _ => {}
                    }
                    i += 1;
                }
                blocks.push(Block::Table {
                    alignments: aligns,
                    header,
                    rows,
                });
            }
            Event::Start(Tag::FootnoteDefinition(id)) => {
                let fid = id.to_string();
                i += 1;
                let mut inner = Vec::new();
                while i < events.len() && !matches!(events[i], Event::End(TagEnd::FootnoteDefinition)) {
                    let slice = &events[i..];
                    let mut sub_blocks = Vec::new();
                    let consumed = convert_single_event(slice, &mut sub_blocks, ctx);
                    inner.extend(sub_blocks);
                    i += consumed;
                }
                blocks.push(Block::FootnoteDef {
                    id: fid,
                    content: inner,
                });
            }
            Event::DisplayMath(latex) => {
                blocks.push(Block::MathDisplay(latex.to_string()));
            }
            Event::Html(html) => {
                let html_str = html.to_string();
                // Check if this is a placeholder
                if let Some(ext) = ctx.find_placeholder(&html_str) {
                    blocks.push(ext.block.clone());
                } else {
                    blocks.push(Block::Html(html_str));
                }
            }
            Event::Rule => {
                blocks.push(Block::ThematicBreak);
            }
            _ => {}
        }
        i += 1;
    }
}

/// Convert a single event (which may span multiple events if it's a container).
/// Returns the number of events consumed.
fn convert_single_event(events: &[Event], blocks: &mut Vec<Block>, ctx: &mut ConvertContext) -> usize {
    if events.is_empty() {
        return 0;
    }

    match &events[0] {
        Event::Start(Tag::Paragraph) => {
            let mut i = 1;
            let children = collect_inlines(events, &mut i);
            // Check for placeholder
            let has_placeholder = children.iter().any(|c| match c {
                Inline::Html(h) | Inline::Text(h) => ctx.find_placeholder(h).is_some(),
                _ => false,
            });
            if has_placeholder {
                for inline in &children {
                    match inline {
                        Inline::Html(h) | Inline::Text(h) => {
                            if let Some(ext) = ctx.find_placeholder(h) {
                                blocks.push(ext.block.clone());
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                blocks.push(Block::Paragraph(children));
            }
            return i + 1;
        }
        Event::Start(Tag::List(_)) | Event::Start(Tag::BlockQuote(_)) => {
            // For nested containers, delegate to the full convert_events
            let mut sub = Vec::new();
            convert_events(&events[..1], &mut sub, ctx);
            blocks.extend(sub);
            // Find the matching end
            let mut depth = 1;
            let mut j = 1;
            while j < events.len() && depth > 0 {
                match &events[j] {
                    Event::Start(_) => depth += 1,
                    Event::End(_) => depth -= 1,
                    _ => {}
                }
                j += 1;
            }
            return j;
        }
        _ => {
            // Non-container event — just skip
            return 1;
        }
    }
}

// ============================================================
// Inline collection helpers
// ============================================================

fn collect_inlines(events: &[Event], i: &mut usize) -> Vec<Inline> {
    let mut inlines = Vec::new();
    // pulldown-cmark splits brackets into separate Text events (e.g. "[", "@key", "]").
    // We accumulate adjacent Text events into a buffer so that split_custom_inlines
    // can see the full "[@key]" and "[[label]]" patterns.
    let mut text_buf = String::new();

    let flush_text = |buf: &mut String, inlines: &mut Vec<Inline>| {
        if !buf.is_empty() {
            split_custom_inlines(buf, inlines);
            buf.clear();
        }
    };

    while *i < events.len() {
        match &events[*i] {
            Event::End(_) => break,

            Event::Text(text) => {
                text_buf.push_str(text);
                *i += 1;
                continue;
            }
            Event::Code(code) => {
                flush_text(&mut text_buf, &mut inlines);
                inlines.push(Inline::Code(code.to_string()));
            }
            Event::InlineMath(math) => {
                flush_text(&mut text_buf, &mut inlines);
                inlines.push(Inline::Math(math.to_string()));
            }
            Event::DisplayMath(math) => {
                // Display math inside a paragraph alongside other content —
                // render as inline with display delimiters since we can't
                // promote it to a block from here.
                flush_text(&mut text_buf, &mut inlines);
                inlines.push(Inline::Html(format!("\\[{}\\]", math)));
            }
            Event::Start(Tag::Strong) => {
                flush_text(&mut text_buf, &mut inlines);
                *i += 1;
                let children = collect_inlines(events, i);
                inlines.push(Inline::Bold(children));
            }
            Event::Start(Tag::Emphasis) => {
                flush_text(&mut text_buf, &mut inlines);
                *i += 1;
                let children = collect_inlines(events, i);
                inlines.push(Inline::Italic(children));
            }
            Event::Start(Tag::Strikethrough) => {
                flush_text(&mut text_buf, &mut inlines);
                *i += 1;
                let children = collect_inlines(events, i);
                inlines.push(Inline::Strikethrough(children));
            }
            Event::Start(Tag::Link { dest_url, title: _, .. }) => {
                flush_text(&mut text_buf, &mut inlines);
                let url = dest_url.to_string();
                *i += 1;
                let children = collect_inlines(events, i);
                inlines.push(Inline::Link { url, children });
            }
            Event::Start(Tag::Image { dest_url, title, .. }) => {
                flush_text(&mut text_buf, &mut inlines);
                let url_str = dest_url.to_string();
                let title_str = title.to_string();
                *i += 1;
                let alt_inlines = collect_inlines(events, i);
                let alt_str = inlines_to_plain_text(&alt_inlines);
                inlines.push(Inline::Image {
                    url: url_str,
                    alt: alt_str,
                    title: title_str,
                });
            }
            Event::FootnoteReference(id) => {
                flush_text(&mut text_buf, &mut inlines);
                inlines.push(Inline::FootnoteRef(id.to_string()));
            }
            Event::SoftBreak => {
                flush_text(&mut text_buf, &mut inlines);
                inlines.push(Inline::SoftBreak);
            }
            Event::HardBreak => {
                flush_text(&mut text_buf, &mut inlines);
                inlines.push(Inline::HardBreak);
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                let html_str = html.as_ref();
                if let Some(inner) = html_str.strip_prefix("<!--XREF:").and_then(|s| s.strip_suffix("-->")) {
                    flush_text(&mut text_buf, &mut inlines);
                    let (label, display) = if let Some(pipe) = inner.find('|') {
                        (inner[..pipe].to_string(), inner[pipe + 1..].to_string())
                    } else {
                        (inner.to_string(), String::new())
                    };
                    inlines.push(Inline::CrossRef {
                        label,
                        display,
                        kind: String::new(),
                        title: String::new(),
                        number: String::new(),
                        content_preview: String::new(),
                    });
                } else if let Some(inner) = html_str.strip_prefix("<!--CITE:").and_then(|s| s.strip_suffix("-->")) {
                    flush_text(&mut text_buf, &mut inlines);
                    // Parse citation(s): @key or @key, locator or @key1; @key2
                    let parts: Vec<&str> = inner.split(';').collect();
                    for (idx, part) in parts.iter().enumerate() {
                        let part = part.trim();
                        let part = part.strip_prefix('@').unwrap_or(part);
                        let (key, locator) = if let Some(comma) = part.find(',') {
                            (part[..comma].trim().to_string(), part[comma + 1..].trim().to_string())
                        } else {
                            (part.trim().to_string(), String::new())
                        };
                        if idx > 0 {
                            inlines.push(Inline::Text(", ".to_string()));
                        }
                        inlines.push(Inline::Citation {
                            key,
                            locator,
                            anchor_id: String::new(),
                            display: String::new(),
                        });
                    }
                } else if let Some(inner) = html_str.strip_prefix("<!--TRCL:").and_then(|s| s.strip_suffix("-->")) {
                    flush_text(&mut text_buf, &mut inlines);
                    inlines.push(Inline::Text(format!("![[{inner}]]")));
                } else {
                    flush_text(&mut text_buf, &mut inlines);
                    inlines.push(Inline::Html(html_str.to_string()));
                }
            }
            _ => {}
        }
        *i += 1;
    }

    flush_text(&mut text_buf, &mut inlines);
    return inlines;
}

/// Split a text string on [[crossref]], [@citation], and ![[transclusion]] syntax.
fn split_custom_inlines(text: &str, inlines: &mut Vec<Inline>) {
    let mut remaining = text;

    while !remaining.is_empty() {
        // Find the next custom syntax marker
        let next_cross_ref = remaining.find("[[");
        let next_citation = remaining.find("[@");

        let next_pos = [next_cross_ref, next_citation].iter().filter_map(|p| *p).min();

        let Some(pos) = next_pos else {
            // No more markers — emit remaining text
            if !remaining.is_empty() {
                inlines.push(Inline::Text(remaining.to_string()));
            }
            return;
        };

        // Emit text before the marker
        if pos > 0 {
            inlines.push(Inline::Text(remaining[..pos].to_string()));
        }

        // Check for transclusion ![[slug]]
        if remaining[pos..].starts_with("[[") && pos > 0 && remaining.as_bytes()[pos - 1] == b'!' {
            // Transclusion — remove the ! we already emitted in the text before
            if let Some(Inline::Text(prev)) = inlines.last_mut()
                && prev.ends_with('!')
            {
                prev.pop();
                if prev.is_empty() {
                    inlines.pop();
                }
            }
            let after = &remaining[pos + 2..];
            if let Some(end) = after.find("]]") {
                let slug = after[..end].to_string();
                // Transclusion is handled at the resolve phase, store as text for now
                inlines.push(Inline::Text(format!("![[{slug}]]")));
                remaining = &after[end + 2..];
                continue;
            }
        }

        // Cross-reference [[label]] or [[label|display]]
        if remaining[pos..].starts_with("[[") {
            let after = &remaining[pos + 2..];
            if let Some(end) = after.find("]]") {
                let ref_content = &after[..end];
                let (label, display) = if let Some(pipe) = ref_content.find('|') {
                    (ref_content[..pipe].to_string(), ref_content[pipe + 1..].to_string())
                } else {
                    (ref_content.to_string(), String::new())
                };
                inlines.push(Inline::CrossRef {
                    label,
                    display,
                    kind: String::new(),
                    title: String::new(),
                    number: String::new(),
                    content_preview: String::new(),
                });
                remaining = &after[end + 2..];
                continue;
            }
        }

        // Citation [@key] or [@key, locator] or [@key1; @key2]
        if remaining[pos..].starts_with("[@") {
            let after = &remaining[pos + 1..]; // after [
            if let Some(bracket_end) = after.find(']') {
                let citation_content = &after[..bracket_end];
                // Split on ; for multi-citations
                let entries: Vec<&str> = citation_content.split(';').collect();
                for (idx, entry) in entries.iter().enumerate() {
                    let entry = entry.trim().strip_prefix('@').unwrap_or(entry.trim());
                    let (key, locator) = if let Some(comma_pos) = entry.find(',') {
                        (
                            entry[..comma_pos].trim().to_string(),
                            entry[comma_pos + 1..].trim().to_string(),
                        )
                    } else {
                        (entry.trim().to_string(), String::new())
                    };
                    if !key.is_empty() {
                        inlines.push(Inline::Citation {
                            key,
                            locator,
                            anchor_id: String::new(),
                            display: String::new(), // resolved later from bibliography
                        });
                        if idx + 1 < entries.len() {
                            inlines.push(Inline::Text("; ".to_string()));
                        }
                    }
                }
                remaining = &after[bracket_end + 1..];
                continue;
            }
        }

        // Couldn't parse — emit the marker chars as text and continue
        inlines.push(Inline::Text(remaining[pos..pos + 2].to_string()));
        remaining = &remaining[pos + 2..];
    }
}

fn collect_raw_text(events: &[Event], i: &mut usize) -> String {
    let mut text = String::new();
    while *i < events.len() {
        match &events[*i] {
            Event::End(_) => break,
            Event::Text(t) => text.push_str(t),
            Event::Code(c) => text.push_str(c),
            Event::SoftBreak | Event::HardBreak => text.push('\n'),
            _ => {}
        }
        *i += 1;
    }
    return text;
}

// ============================================================
// Utility helpers
// ============================================================

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn heading_id_from_inlines(inlines: &[Inline]) -> String {
    let plain = inlines_to_plain_text(inlines);
    return plain
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
}

pub fn inlines_to_plain_text(inlines: &[Inline]) -> String {
    let mut text = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => text.push_str(t),
            Inline::Code(c) => text.push_str(c),
            Inline::Math(m) => text.push_str(m),
            Inline::Bold(children) | Inline::Italic(children) | Inline::Strikethrough(children) => {
                text.push_str(&inlines_to_plain_text(children));
            }
            Inline::Link { children, .. } => {
                text.push_str(&inlines_to_plain_text(children));
            }
            Inline::CrossRef { label, display, .. } => {
                if !display.is_empty() {
                    text.push_str(display);
                } else {
                    text.push_str(label);
                }
            }
            Inline::SoftBreak | Inline::HardBreak => text.push(' '),
            _ => {}
        }
    }
    return text;
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_paragraph_with_math() {
        let body = "A group $(G, \\cdot)$ is a set.";
        let (blocks, _) = parse_markdown(body, false);
        assert_eq!(blocks.len(), 1, "Expected 1 block, got: {blocks:#?}");
        match &blocks[0] {
            Block::Paragraph(inlines) => {
                assert!(inlines.len() >= 3, "Expected at least 3 inlines, got: {inlines:#?}");
                assert!(matches!(&inlines[0], Inline::Text(t) if t.contains("group")));
                assert!(matches!(&inlines[1], Inline::Math(m) if m.contains("G")));
            }
            other => panic!("Expected Paragraph, got: {other:#?}"),
        }
    }

    #[test]
    fn test_definition_block_with_numbering() {
        let body = "```definition Group {#def:group}\nA **group** is a set.\n```";
        let (blocks, metas) = parse_markdown(body, false);
        assert_eq!(blocks.len(), 1, "Expected 1 block, got: {blocks:#?}");
        match &blocks[0] {
            Block::LabeledBlock {
                kind,
                id,
                number,
                title,
                ..
            } => {
                assert_eq!(kind, "definition");
                assert_eq!(id, "def-group");
                assert_eq!(number, "1");
                assert_eq!(title, "Group");
            }
            other => panic!("Expected LabeledBlock, got: {other:#?}"),
        }
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].number, "1");
    }

    #[test]
    fn test_citation_inline() {
        let body = "See Lang [@lang_algebra] for details.";
        let (blocks, _) = parse_markdown(body, false);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Paragraph(inlines) => {
                let has_citation = inlines
                    .iter()
                    .any(|i| matches!(i, Inline::Citation { key, .. } if key == "lang_algebra"));
                assert!(has_citation, "Expected citation inline, got: {inlines:#?}");
            }
            other => panic!("Expected Paragraph, got: {other:#?}"),
        }
    }

    #[test]
    fn test_cross_reference() {
        let body = "See [[def:group]] for the definition.";
        let (blocks, _) = parse_markdown(body, false);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Paragraph(inlines) => {
                let has_xref = inlines
                    .iter()
                    .any(|i| matches!(i, Inline::CrossRef { label, .. } if label == "def:group"));
                assert!(has_xref, "Expected cross-reference, got: {inlines:#?}");
            }
            other => panic!("Expected Paragraph, got: {other:#?}"),
        }
    }

    #[test]
    fn test_display_math() {
        let body = "$$\\varphi(a) = b$$";
        let (blocks, _) = parse_markdown(body, false);
        let has_display_math = blocks.iter().any(|b| matches!(b, Block::MathDisplay(_)));
        assert!(has_display_math, "Expected MathDisplay block, got: {blocks:#?}");
    }

    #[test]
    fn test_equation_block() {
        let body = "```equation First Iso {#eq:first-iso}\nG / \\ker\\varphi \\cong \\text{im}\\,\\varphi\n```";
        let (blocks, metas) = parse_markdown(body, false);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Equation {
                id,
                number,
                title,
                latex,
                ..
            } => {
                assert_eq!(id, "eq-first-iso");
                assert_eq!(number, "1");
                assert_eq!(title, "First Iso");
                assert!(latex.contains("ker"));
            }
            other => panic!("Expected Equation, got: {other:#?}"),
        }
        assert_eq!(metas[0].number, "1");
    }

    #[test]
    fn test_mermaid_diagram() {
        let body = "```mermaid\ngraph TD\n    A --> B\n```";
        let (blocks, _) = parse_markdown(body, false);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Diagram { variant, source, .. } => {
                assert!(matches!(variant, DiagramKind::Mermaid));
                assert!(source.contains("graph TD"));
            }
            other => panic!("Expected Diagram, got: {other:#?}"),
        }
    }

    #[test]
    fn test_list_with_math() {
        let body = "- Item $x$\n- Item $y$";
        let (blocks, _) = parse_markdown(body, false);
        assert_eq!(blocks.len(), 1, "Expected 1 block, got: {blocks:#?}");
        match &blocks[0] {
            Block::List { items, .. } => {
                assert_eq!(items.len(), 2, "Expected 2 items, got: {items:#?}");
            }
            other => panic!("Expected List, got: {other:#?}"),
        }
    }

    #[test]
    fn test_multiple_definitions_numbered() {
        let body = "```definition Group {#def:group}\nContent A.\n```\n\n```definition Subgroup \
                    {#def:subgroup}\nContent B.\n```\n\n```theorem Lagrange {#thm:lagrange}\nContent C.\n```";
        let (blocks, metas) = parse_markdown(body, false);
        assert_eq!(blocks.len(), 3, "Expected 3 blocks, got: {blocks:#?}");
        assert_eq!(metas.len(), 3);
        assert_eq!(metas[0].number, "1");
        assert_eq!(metas[1].number, "2");
        assert_eq!(metas[2].number, "3");
    }

    #[test]
    fn test_definition_content_has_math() {
        let body = "```definition Group {#def:group}\nA **group** $(G, \\cdot)$ is a set $G$ with an operation.\n```";
        let (blocks, _) = parse_markdown(body, false);
        match &blocks[0] {
            Block::LabeledBlock { content, .. } => match &content[0] {
                Block::Paragraph(inlines) => {
                    let has_bold = inlines.iter().any(|i| matches!(i, Inline::Bold(_)));
                    let has_math = inlines.iter().any(|i| matches!(i, Inline::Math(_)));
                    assert!(has_bold, "Expected bold in definition content: {inlines:#?}");
                    assert!(has_math, "Expected math in definition content: {inlines:#?}");
                }
                other => panic!("Expected Paragraph in content, got: {other:#?}"),
            },
            other => panic!("Expected LabeledBlock, got: {other:#?}"),
        }
    }

    #[test]
    fn test_ordered_list_with_inline_math() {
        let body = "Text before:\n\n1. $a(bv) = (ab)v$ for all $a, b \\in F$\n2. $1v = v$ for all $v \\in V$\n3. $a(u \
                    + v) = au + av$\n4. $(a + b)v = av + bv$";
        let (blocks, _) = parse_markdown(body, false);
        eprintln!("BLOCKS: {blocks:#?}");
        // Find the list block
        let list = blocks.iter().find(|b| matches!(b, Block::List { .. }));
        assert!(list.is_some(), "Expected a List block, got: {blocks:#?}");
        if let Some(Block::List { items, .. }) = list {
            assert_eq!(items.len(), 4, "Expected 4 items, got: {items:#?}");
            // Each item should have content (not be empty)
            for (i, item) in items.iter().enumerate() {
                assert!(!item.is_empty(), "Item {i} is empty!");
                // Check the paragraph inside has inlines
                if let Some(Block::Paragraph(inlines)) = item.first() {
                    assert!(!inlines.is_empty(), "Item {i} paragraph has no inlines: {item:#?}");
                }
            }
        }
    }

    #[test]
    fn test_footnote() {
        let body = "Text with footnote.[^1]\n\n[^1]: This is the footnote.";
        let (blocks, _) = parse_markdown(body, false);
        let has_ref = blocks.iter().any(|b| match b {
            Block::Paragraph(inlines) => inlines.iter().any(|i| matches!(i, Inline::FootnoteRef(_))),
            _ => false,
        });
        let has_def = blocks.iter().any(|b| matches!(b, Block::FootnoteDef { .. }));
        assert!(has_ref, "Expected footnote reference, got: {blocks:#?}");
        assert!(has_def, "Expected footnote definition, got: {blocks:#?}");
    }

    #[test]
    fn test_display_math_with_eq_label_next_line() {
        let body = "$$E = mc^2$$\n{#eq:mass-energy}";
        let (blocks, metas) = parse_markdown(body, false);
        eprintln!("blocks: {blocks:#?}");
        eprintln!("metas: {metas:#?}");
        assert_eq!(metas.len(), 1, "Expected 1 meta, got: {metas:#?}");
        assert_eq!(metas[0].label, "eq:mass-energy");
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b, Block::Equation { label, .. } if label == "eq:mass-energy")),
            "Expected Equation block, got: {blocks:#?}"
        );
    }

    #[test]
    fn test_display_math_with_eq_label_inside_block() {
        let body = "```definition Test {#def:test}\nSome text.\n\n$$x + y = z$$\n{#eq:test-eq}\n\nMore text.\n```";
        let (blocks, metas) = parse_markdown(body, false);
        eprintln!("blocks: {blocks:#?}");
        eprintln!("metas: {metas:#?}");
        // The definition block should exist
        assert!(
            blocks.iter().any(|b| matches!(b, Block::LabeledBlock { .. })),
            "Expected LabeledBlock"
        );
        // The equation should be nested inside
        if let Some(Block::LabeledBlock { content, .. }) = blocks.first() {
            assert!(
                content
                    .iter()
                    .any(|b| matches!(b, Block::Equation { label, .. } if label == "eq:test-eq")),
                "Expected Equation inside LabeledBlock, got: {content:#?}"
            );
        }
    }
}
