# Content Authoring Guide

Available features for markdown posts and TOML data files.

## Markdown Posts

Posts live in `content/posts/` as `.md` files. Subdirectories create nested slugs
(e.g. `content/posts/cs/sorting-algorithms.md` → `/blog/cs/sorting-algorithms`).

### Frontmatter

Every post starts with YAML-style frontmatter between `---` markers.
Fields should follow this standard order:

```
---
title: My Post Title
description: A brief summary shown in the post list
series: My Series Name
series_order: 1
tags: rust, wasm, leptos
publication: cs
project: lsck0.github.io
sources: https://example.com/ref1, https://example.com/ref2
date: 2026-03-15
draft: true
toc: true
---
```

#### Required fields

| Field   | Description                   |
| ------- | ----------------------------- |
| `title` | Display title of the post     |
| `date`  | Publication date (YYYY-MM-DD) |

#### Optional fields

| Field          | Description                                                   |
| -------------- | ------------------------------------------------------------- |
| `description`  | Short summary shown under the title in the blog listing       |
| `series`       | Name of a series — posts with the same name are grouped       |
| `series_order` | Integer ordering within the series (ascending)                |
| `tags`         | Comma-separated tags, used for filtering on `/blog`           |
| `project`      | Links this post to a project entry                            |
| `publication`  | Links this post to a publication entry                        |
| `sources`      | Comma-separated URLs for references not linked in the body    |
| `draft`        | Set to `true` to show only in dev mode with DRAFT banner      |
| `toc`          | Set to `true` to show a collapsible table of contents         |

### Content features

#### Code blocks

Fenced code blocks with a language tag get syntax highlighting via Prism.
A "copy code" button appears on hover (top-right).

````
```rust
fn main() { println!("hello"); }
```
````

#### Diff highlighting

Use `diff` as the language for code blocks. Lines starting with `+` get a green
background, lines starting with `-` get red:

````
```diff
+added line
-removed line
 unchanged line
```
````

#### Special code blocks

| Language  | Renders as                    | Copy button    |
| --------- | ----------------------------- | -------------- |
| `tikz`    | TikZ diagram via tikzjax      | "copy tex"     |
| `tikzcd`  | Commutative diagram (tikz-cd) | "copy tex"     |
| `mermaid` | Mermaid diagram               | "copy mermaid" |

All special blocks show a copy button on hover that copies the source.

#### Math

Inline math with `$...$` and display math with `$$...$$`. Rendered via KaTeX.
Display math blocks show a "copy tex" button on hover.

#### Labeled blocks (definitions, theorems, etc.)

Use backtick-fenced blocks where the "language" is a block kind:

````
```definition Group {#def-group}
A **group** is a set $G$ with a binary operation $\cdot$ satisfying...
```

```theorem Lagrange's Theorem {#thm-lagrange}
For any finite group $G$ and subgroup $H$...
```

```proof
By counting cosets...
```
````

Supported numbered kinds: `definition`, `theorem`, `lemma`, `corollary`,
`proposition`, `example`, `axiom`, `remark`, `conjecture`, `exercise`, `problem`.

Unnumbered: `proof` (with QED symbol).

Callouts: `tip`, `warning`, `danger`, `note`, `info`.

All numbered blocks share a single counter per post. Labels are optional but
enable cross-referencing.

##### Definition aliases

Definitions can have aliases that also auto-link in prose:

````
```definition Characteristic | char {#def:characteristic}
The characteristic of a field is...
```
````

Both "characteristic" and "char" will auto-link to this definition.

##### Nesting

Use more backticks for the outer block to nest blocks:

`````
````theorem Fundamental Theorem {#fund-thm}
Statement.

```proof
Proof here.
```
````
`````

#### Cross-references

Reference labeled blocks from any post using `[[label]]`:

```
By [[def-group]], we know that...
See [[thm-lagrange]] for details.
```

Cross-post references with explicit slug: `[[math/group-theory#def-group]]`.

Cross-references render as links with hover previews showing the block content
(with KaTeX math rendering in the tooltip). Custom display text:
`[[def-group|the definition]]`.

#### Footnotes

Standard markdown footnotes:

```
Some text[^1] with a footnote.

[^1]: The footnote content.
```

On wide screens (≥1200px), footnotes float into the right margin as sidenotes.

#### Callouts / Admonitions

Use backtick-fenced blocks with a callout kind, or blockquote syntax:

````
```tip
This is a tip callout.
```

```warning
This is a warning.
```
````

Or blockquote syntax:

```
> [!tip]
> This is a tip callout.
```

Supported types: `tip` (green), `warning` (yellow), `danger` (red),
`note` (accent), `info` (blue).

#### Transclusion

Embed the body of another post inline using `![[slug]]`:

```
![[cs/lambda-calculus]]
```

The referenced post's body is inserted at build time.

#### Internal references

Link to other posts using `/blog/{slug}`. The post page automatically shows:

- **Internal references** — posts this post links to, with backlink markers
- **External references** — external URLs linked in the body
- **Sources** — URLs listed in the `sources` frontmatter field
- **Referenced internally by** — other posts that link to this post

Each reference includes backlink markers (↑1, ↑2, ...) that scroll to where
the link appears in the body.

#### Link hover previews

- **Internal links**: show title, description, tags, and series info
- **Cross-references**: show the block content with rendered math
- **External links**: show favicon, domain, and URL path

#### Series navigation

Posts with the same `series` frontmatter value get a collapsible series nav box
with a table of contents and prev/next links, ordered by `series_order`.

The blog listing shows a series badge (e.g. "[Series Name 2/4]") next to each post.

#### Searchable content (Ctrl+F)

Rendered KaTeX math, Mermaid diagrams, and TikZ diagrams include hidden text
copies of their source code, making them findable via browser Ctrl+F search.

### Blog listing features

- **Fuzzy search** — subsequence matching via nucleo-matcher (title 3× weight)
- **Tag filtering** — click to cycle: neutral → include → exclude
- **Bookmarks filter** — show only bookmarked posts
- **Read/unread** — posts are marked "read" via localStorage when visited
- **Series badge** — shows "[Series Name X/Y]" for series posts
- **Pagination** — 10 posts per page
- **View modes** — list, tree (folder structure), series (grouped by series with progress), bookmarks, graph (force-directed knowledge graph)
- **Post count** — shows filtered/total count when filters are active

### Post page features

- **Bookmark button** — SVG bookmark icon before the title
- **Reading progress bar** — thin accent bar at viewport top
- **In-post search** — Ctrl+Shift+F to search within the post content, with match highlighting and prev/next navigation
- **Table of contents** — collapsible, auto-generated from h1/h2/h3 headings (opt-in via `toc: true`)
- **Draft banner** — yellow "DRAFT" banner for draft posts (dev mode only)
- **Scroll-to-top** — appears after scrolling past 50% viewport height
- **Giscus comments** — powered by GitHub Discussions
- **Read tracking** — marks post as read in localStorage

## Site Metadata (`content/meta.toml`)

Site-wide metadata for SEO and page titles:

```toml
[site]
title = "/dev/lsck0"
description = "computer science, mathematics, and software engineering"
author = "Luca Sandrock"
url = "https://lsck0.github.io"

[pages.home]
title = "/dev/lsck0"
description = "Personal knowledge base."

[pages.blog]
title = "blog"
description = "Posts on CS, math, and engineering."
```

Each page section provides a title and description used for `<title>` tags
and meta descriptions.

## Build Pipeline

The Makefile.toml defines three tasks:

| Task       | Command                                           | Description                                    |
| ---------- | ------------------------------------------------- | ---------------------------------------------- |
| `dev`      | `trunk serve --port 3000 --open`                  | Development server with HMR                    |
| `build`    | `trunk build --release` + wasm-opt + indexer + 404 | Production build (single source of truth)      |
| `ci`       | clippy + fmt check + `makers build`               | CI validation                                  |
| `wasm-opt` | wasm-opt with bulk-memory flags                   | Manual WASM optimization (called by `build`)   |

## Build-time Indexer

Running `cargo run --package indexer` after `trunk build` generates:

| File                           | Description                                  |
| ------------------------------ | -------------------------------------------- |
| `dist/graph.json`              | Node/edge graph of posts and their relations |
| `dist/search_index.json`       | Search index with slug, title, tags, desc    |
| `dist/rss.xml`                 | RSS 2.0 feed                                 |
| `dist/atom.xml`                | Atom feed                                    |
| `dist/sitemap.xml`             | Sitemap for search engines                   |
| `dist/llms.txt`                | AI scraping opt-out file                     |
| `dist/blog/{slug}/index.html`  | Per-post HTML with OG meta tags for embeds   |

The OG pages are copies of `index.html` with `<meta property="og:*">` and
`<meta name="twitter:*">` tags injected, so social platform crawlers see correct
metadata even though the site is client-side rendered.

## Client State

Bookmark and read state is managed via localStorage through a centralized
`components/storage` module. All pages use the same module — no duplicated
localStorage access patterns.

| Key pattern      | Value     | Description                |
| ---------------- | --------- | -------------------------- |
| `bookmark:{slug}` | `"1"`     | Post is bookmarked         |
| `read:{slug}`     | timestamp | Post has been viewed        |
| `theme`           | `"dark"` / `"light"` | Current theme  |

## Content Pipeline Architecture

The content pipeline has three layers:

1. **`crates/content/`** — Shared library for frontmatter parsing, link extraction,
   and transclusion. Used by both the proc macro and the indexer.

2. **`crates/macros/`** — Proc macro crate that reads content at compile time,
   extracts labeled blocks, resolves cross-references, and emits a static
   `&[Post]` array embedded in the WASM binary.

3. **`crates/indexer/`** — Post-build binary that generates feeds, search index,
   graph data, OG pages, and sitemap from the same content source.

## Projects (`content/projects.toml`)

```toml
[[projects]]
title = "my-project"
description = "What it does."
url = "https://github.com/..."
status = "maintained"
```

### Fields

| Field         | Type                | Description                                         |
| ------------- | ------------------- | --------------------------------------------------- |
| `title`       | string or segment[] | Project name                                        |
| `description` | string or segment[] | Short description                                   |
| `url`         | string              | Link (empty string for no link)                     |
| `status`      | string              | One of: `maintained`, `wip`, `planned`, `abandoned` |

### Text segments

Fields like `title` and `description` support a scramble effect for teasers:

```toml
title = ["type-", { scrambled = 8 }]
description = [{ scrambled = 50 }]
```

- Plain string: rendered as-is
- Array of strings/tables: mixed plain + scrambled text
- `{ scrambled = N }`: renders N characters of randomly cycling text

## Publications (`content/publications.toml`)

```toml
[[publications]]
title = "Paper Title"
description = "Venue, year, and summary."
url = "https://arxiv.org/..."
authors = "A. Author, B. Author"
date = "2026"
```

### Fields

| Field         | Type                | Description                    |
| ------------- | ------------------- | ------------------------------ |
| `title`       | string or segment[] | Publication title              |
| `description` | string or segment[] | Venue/summary (supports LaTeX) |
| `url`         | string              | Link to the paper              |
| `authors`     | string or segment[] | Author list                    |
| `date`        | string or segment[] | Publication year               |

All text fields support the same segment array syntax as projects.

## Anti-AI Measures

- `robots.txt` blocks known AI crawlers (ClaudeBot, GPTBot, etc.)
- `<meta name="robots" content="noai, noimageai">` in HTML head
- `/llms.txt` generated by the indexer
- TOS explicitly opts out of AI training (MIT license with AI carve-out)
