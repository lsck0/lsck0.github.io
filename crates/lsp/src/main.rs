#![allow(clippy::needless_return)]

//! Language Server for the blog's markdown content files.
//!
//! Provides:
//! - Autocomplete for `[[cross-references]]` and `[@citations]`
//! - Go-to-definition for cross-references
//! - Diagnostics for broken references

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

use tower_lsp::{Client, LanguageServer, LspService, Server, jsonrpc::Result, lsp_types::*};

// ============================================================
// Content index
// ============================================================

/// Pre-built index of all available labels, citations, and post slugs.
struct ContentIndex {
    /// label → (kind, title, slug)
    labels: HashMap<String, (String, String, String)>,
    /// bib key → display label
    citations: HashMap<String, String>,
    /// slug → title
    posts: HashMap<String, String>,
}

impl ContentIndex {
    fn new() -> Self {
        return Self {
            labels: HashMap::new(),
            citations: HashMap::new(),
            posts: HashMap::new(),
        };
    }

    fn rebuild(&mut self, content_dir: &Path) {
        self.labels.clear();
        self.citations.clear();
        self.posts.clear();

        // Scan posts directory
        let posts_dir = content_dir.join("posts");
        if posts_dir.exists() {
            self.scan_posts(&posts_dir, &posts_dir);
        }

        // Parse bibliography
        let bib_path = content_dir.join("references.bib");
        if bib_path.exists() {
            let entries = ir::bib::parse_bib_file(&bib_path);
            for (key, entry) in entries {
                self.citations.insert(key, entry.citation_label());
            }
        }
    }

    fn scan_posts(&mut self, base: &PathBuf, dir: &PathBuf) {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_posts(base, &path);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                self.index_post(base, &path);
            }
        }
    }

    fn index_post(&mut self, base: &PathBuf, path: &PathBuf) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let relative = path.strip_prefix(base).unwrap_or(path).with_extension("");
        let slug = relative.to_str().unwrap_or("").replace('\\', "/");

        let (metadata, body) = ir::frontmatter::parse_frontmatter(&content);
        let title = metadata.get("title").cloned().unwrap_or_else(|| slug.clone());

        self.posts.insert(slug.clone(), title);

        let has_toc = ir::frontmatter::meta_is(&metadata, "toc", "true");
        let (_, block_metas) = ir::parse::parse_markdown(&body, has_toc);

        for meta in block_metas {
            if !meta.label.is_empty() {
                self.labels.insert(
                    meta.label.clone(),
                    (meta.kind.clone(), meta.title.clone(), slug.clone()),
                );
            }
        }
    }
}

// ============================================================
// LSP backend
// ============================================================

struct Backend {
    client: Client,
    index: Mutex<ContentIndex>,
    content_dir: Mutex<Option<PathBuf>>,
}

impl Backend {
    fn rebuild_index(&self) {
        let content_dir = self.content_dir.lock().unwrap().clone();
        if let Some(dir) = content_dir {
            self.index.lock().unwrap().rebuild(&dir);
        }
    }

    fn find_content_dir(root: &Url) -> Option<PathBuf> {
        let root_path = root.to_file_path().ok()?;
        let content_dir = root_path.join("content");
        if content_dir.exists() {
            return Some(content_dir);
        }
        return None;
    }

    fn completion_items(&self, trigger: &str) -> Vec<CompletionItem> {
        let index = self.index.lock().unwrap();
        let mut items = Vec::new();

        match trigger {
            "[[" => {
                // Cross-reference completions: labels and posts
                for (label, (kind, title, slug)) in &index.labels {
                    let detail = if title.is_empty() {
                        format!("{kind} in {slug}")
                    } else {
                        format!("{kind}: {title} ({slug})")
                    };
                    items.push(CompletionItem {
                        label: label.clone(),
                        kind: Some(CompletionItemKind::REFERENCE),
                        detail: Some(detail),
                        insert_text: Some(format!("{label}]]")),
                        ..Default::default()
                    });
                }
                // Post slug completions for cross-post refs
                for (slug, title) in &index.posts {
                    items.push(CompletionItem {
                        label: slug.clone(),
                        kind: Some(CompletionItemKind::FILE),
                        detail: Some(title.clone()),
                        insert_text: Some(format!("{slug}#]]")),
                        ..Default::default()
                    });
                }
            }
            "[@" => {
                // Citation completions
                for (key, display) in &index.citations {
                    items.push(CompletionItem {
                        label: key.clone(),
                        kind: Some(CompletionItemKind::TEXT),
                        detail: Some(display.clone()),
                        insert_text: Some(format!("{key}]")),
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }

        return items;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Detect content directory from workspace root
        if let Some(root_uri) = params.root_uri
            && let Some(content_dir) = Self::find_content_dir(&root_uri)
        {
            *self.content_dir.lock().unwrap() = Some(content_dir);
            self.rebuild_index();
        }

        return Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["[".to_string(), "@".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "blog-lsp initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        return Ok(());
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        if !uri.path().ends_with(".md") {
            return Ok(None);
        }

        let position = params.text_document_position.position;

        // Read the current document to detect trigger context
        let trigger = params.context.and_then(|ctx| ctx.trigger_character).unwrap_or_default();

        // Check what the user is typing based on trigger
        let items = if trigger == "@" || trigger == "[" {
            // Try to determine if we're in [[ or [@ context
            // We need to look at the text before the cursor
            let file_path = uri.to_file_path().ok();
            let context_trigger = if let Some(path) = file_path {
                detect_trigger_context(&path, position)
            } else {
                None
            };

            match context_trigger.as_deref() {
                Some("[[") => self.completion_items("[["),
                Some("[@") => self.completion_items("[@"),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        if items.is_empty() {
            return Ok(None);
        }
        return Ok(Some(CompletionResponse::Array(items)));
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let file_path = match uri.to_file_path() {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let label = match extract_reference_at_position(&file_path, position) {
            Some(l) => l,
            None => return Ok(None),
        };

        let index = self.index.lock().unwrap();
        if let Some((_, _, slug)) = index.labels.get(&label) {
            let content_dir = self.content_dir.lock().unwrap().clone();
            if let Some(dir) = content_dir {
                let target_path = dir.join("posts").join(format!("{slug}.md"));
                if target_path.exists()
                    && let Ok(target_uri) = Url::from_file_path(&target_path)
                {
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                        uri: target_uri,
                        range: Range::default(),
                    })));
                }
            }
        }

        return Ok(None);
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        // Rebuild index on save
        self.rebuild_index();
        self.client
            .log_message(MessageType::INFO, "blog-lsp: index rebuilt")
            .await;
    }

    async fn did_open(&self, _: DidOpenTextDocumentParams) {}
    async fn did_change(&self, _: DidChangeTextDocumentParams) {}
}

// ============================================================
// Text analysis helpers
// ============================================================

fn detect_trigger_context(path: &PathBuf, position: Position) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let line = content.lines().nth(position.line as usize)?;
    let col = position.character as usize;
    let prefix = &line[..col.min(line.len())];

    if prefix.ends_with("[@") || prefix.ends_with("[@ ") {
        return Some("[@".to_string());
    }
    if prefix.ends_with("[[") {
        return Some("[[".to_string());
    }
    // Check if we're already inside [@ or [[
    if let Some(bracket_pos) = prefix.rfind("[@") {
        let between = &prefix[bracket_pos..];
        if !between.contains(']') {
            return Some("[@".to_string());
        }
    }
    if let Some(bracket_pos) = prefix.rfind("[[") {
        let between = &prefix[bracket_pos..];
        if !between.contains("]]") {
            return Some("[[".to_string());
        }
    }
    return None;
}

fn extract_reference_at_position(path: &PathBuf, position: Position) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let line = content.lines().nth(position.line as usize)?;
    let col = position.character as usize;

    // Find [[...]] around the cursor
    let before = &line[..col.min(line.len())];
    let after = &line[col.min(line.len())..];

    if let Some(open) = before.rfind("[[")
        && let Some(close) = after.find("]]")
    {
        let label_start = open + 2;
        let label = format!("{}{}", &before[label_start..], &after[..close]);
        let label = label.split('|').next().unwrap_or(&label).to_string();
        return Some(label);
    }

    return None;
}

// ============================================================
// Entry point
// ============================================================

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        index: Mutex::new(ContentIndex::new()),
        content_dir: Mutex::new(None),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
