#![allow(clippy::needless_return)]

//! BibTeX parser and citation label formatting.
//!
//! Parses `.bib` files into structured entries at compile time.
//! Supports `@book`, `@article`, `@inproceedings`, `@misc`, and other entry types.

use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};

// ============================================================
// Data types
// ============================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BibEntry {
    pub key: String,
    pub entry_type: String,
    pub fields: HashMap<String, String>,
}

impl BibEntry {
    pub fn field(&self, name: &str) -> &str {
        return self.fields.get(name).map(|v| v.as_str()).unwrap_or("");
    }

    pub fn author(&self) -> &str {
        return self.field("author");
    }

    pub fn title(&self) -> &str {
        return self.field("title");
    }

    pub fn year(&self) -> &str {
        return self.field("year");
    }

    pub fn url(&self) -> &str {
        let url = self.field("url");
        if !url.is_empty() {
            return url;
        }
        let doi = self.field("doi");
        if !doi.is_empty() && doi.starts_with("http") {
            return self.fields.get("doi").map(|v| v.as_str()).unwrap_or("");
        }
        return "";
    }

    // ---- citation label formatting ----

    /// Short author label: "Lang" for one, "Dummit and Foote" for two,
    /// "Cohen et al." for three or more.
    pub fn short_author(&self) -> String {
        let raw = self.author();
        if raw.is_empty() {
            return self.key.clone();
        }
        let authors: Vec<&str> = raw.split(" and ").collect();
        let last_names: Vec<String> = authors.iter().map(|a| extract_last_name(a.trim())).collect();
        match last_names.len() {
            0 => self.key.clone(),
            1 => last_names[0].clone(),
            2 => format!("{} and {}", last_names[0], last_names[1]),
            _ => format!("{} et al.", last_names[0]),
        }
    }

    /// Citation label in AMS alpha style (e.g. "Lan02", "DF04", "CFW21").
    pub fn citation_label(&self) -> String {
        let raw = self.author();
        if raw.is_empty() {
            return self.key.clone();
        }
        let authors: Vec<&str> = raw.split(" and ").collect();
        let last_names: Vec<String> = authors.iter().map(|a| extract_last_name(a.trim())).collect();

        let year = self.year();
        let year_suffix: String = if year.len() >= 2 {
            year[year.len() - 2..].to_string()
        } else {
            year.to_string()
        };

        let prefix = match last_names.len() {
            0 => return self.key.clone(),
            1 => {
                // Single author: first 3 letters of last name
                let name = &last_names[0];
                name.chars().take(3).collect::<String>()
            }
            2..=4 => {
                // 2-4 authors: first letter of each
                last_names
                    .iter()
                    .map(|n| n.chars().next().unwrap_or('?'))
                    .collect::<String>()
            }
            _ => {
                // 5+ authors: first letter of first 3 + "+"
                let letters: String = last_names
                    .iter()
                    .take(3)
                    .map(|n| n.chars().next().unwrap_or('?'))
                    .collect();
                format!("{letters}+")
            }
        };

        return format!("{prefix}{year_suffix}");
    }

    // ---- bibliography entry formatting ----

    /// Format this entry as an HTML bibliography line in AMS style.
    ///
    /// Books:  Author, *Title*, Publisher, Place, Year.
    /// Articles: Author, Title, *Journal* **vol**(num):pages, Year.
    /// Inproceedings: Author, Title, in *Booktitle*, pages, Publisher, Year.
    pub fn format_html(&self) -> String {
        let mut parts = Vec::new();

        let author = self.author();
        if !author.is_empty() {
            parts.push(format_authors_full(author));
        }

        let title = self.title();
        let year = self.year();

        match self.entry_type.as_str() {
            "article" => {
                // AMS article: Author, Title, *Journal* **vol**(num):pages, year.
                if !title.is_empty() {
                    parts.push(strip_bib_braces(title));
                }
                let journal = self.field("journal");
                if !journal.is_empty() {
                    let mut venue = format!("<em>{}</em>", strip_bib_braces(journal));
                    let vol = self.field("volume");
                    if !vol.is_empty() {
                        venue.push_str(&format!(" <strong>{vol}</strong>"));
                        let number = self.field("number");
                        if !number.is_empty() {
                            venue.push_str(&format!("({number})"));
                        }
                    }
                    let pages = self.field("pages");
                    if !pages.is_empty() {
                        venue.push_str(&format!(":{}", pages.replace("--", "\u{2013}")));
                    }
                    parts.push(venue);
                }
                if !year.is_empty() {
                    parts.push(year.to_string());
                }
            }
            "inproceedings" => {
                // AMS inproceedings: Author, Title, in *Booktitle*, pages, Publisher, Year.
                if !title.is_empty() {
                    parts.push(strip_bib_braces(title));
                }
                let booktitle = self.field("booktitle");
                if !booktitle.is_empty() {
                    parts.push(format!("in <em>{}</em>", strip_bib_braces(booktitle)));
                }
                let pages = self.field("pages");
                if !pages.is_empty() {
                    parts.push(format!("pp. {}", pages.replace("--", "\u{2013}")));
                }
                let publisher = self.field("publisher");
                if !publisher.is_empty() {
                    parts.push(strip_bib_braces(publisher));
                }
                if !year.is_empty() {
                    parts.push(year.to_string());
                }
            }
            _ => {
                // AMS book/misc: Author, *Title*, Publisher, Place, Year.
                if !title.is_empty() {
                    parts.push(format!("<em>{}</em>", strip_bib_braces(title)));
                }
                let series = self.field("series");
                let volume = self.field("volume");
                if !series.is_empty() && !volume.is_empty() {
                    parts.push(format!("vol. {volume} of <em>{}</em>", strip_bib_braces(series)));
                } else if !series.is_empty() {
                    parts.push(strip_bib_braces(series));
                }
                let publisher = self.field("publisher");
                if !publisher.is_empty() {
                    parts.push(strip_bib_braces(publisher));
                }
                let address = self.field("address");
                if !address.is_empty() {
                    parts.push(strip_bib_braces(address));
                }
                if !year.is_empty() {
                    parts.push(year.to_string());
                }
            }
        }

        let mut html = parts.join(", ");
        if !html.is_empty() && !html.ends_with('.') {
            html.push('.');
        }

        // Append URL/DOI/arXiv link
        let url = self.url();
        if !url.is_empty() {
            html.push_str(&format!(
                " <a href=\"{url}\" target=\"_blank\" rel=\"noopener\">[link]</a>"
            ));
        } else {
            let doi = self.field("doi");
            if !doi.is_empty() {
                let doi_url = if doi.starts_with("http") {
                    doi.to_string()
                } else {
                    format!("https://doi.org/{doi}")
                };
                html.push_str(&format!(
                    " <a href=\"{doi_url}\" target=\"_blank\" rel=\"noopener\">[doi]</a>"
                ));
            }
            let eprint = self.field("eprint");
            if !eprint.is_empty() {
                html.push_str(&format!(
                    " <a href=\"https://arxiv.org/abs/{eprint}\" target=\"_blank\" rel=\"noopener\">[arXiv]</a>"
                ));
            }
        }

        return html;
    }
}

// ============================================================
// Parsing
// ============================================================

/// Parse a `.bib` file from disk.
pub fn parse_bib_file(path: &Path) -> HashMap<String, BibEntry> {
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("Failed to read references file {}: {e}", path.display());
    });
    return parse_bib_str(&content);
}

/// Parse a BibTeX string into a map of key → BibEntry.
pub fn parse_bib_str(input: &str) -> HashMap<String, BibEntry> {
    let mut entries = HashMap::new();
    let mut pos = 0;
    let bytes = input.as_bytes();

    while pos < bytes.len() {
        match input[pos..].find('@') {
            Some(offset) => pos += offset,
            None => break,
        }

        pos += 1; // skip @
        let type_end = input[pos..].find('{').unwrap_or(input.len() - pos);
        let entry_type = input[pos..pos + type_end].trim().to_lowercase();
        pos += type_end + 1; // skip past {

        // Skip meta entries
        if entry_type == "comment" || entry_type == "preamble" || entry_type == "string" {
            let mut depth = 1;
            while pos < bytes.len() && depth > 0 {
                match bytes[pos] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                pos += 1;
            }
            continue;
        }

        // Citation key
        let key_end = input[pos..].find(',').unwrap_or(input.len() - pos);
        let key = input[pos..pos + key_end].trim().to_string();
        pos += key_end + 1;

        // Fields
        let mut fields = HashMap::new();
        loop {
            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos >= bytes.len() || bytes[pos] == b'}' {
                pos += 1;
                break;
            }

            // Line comments inside entry
            if pos + 1 < bytes.len() && bytes[pos] == b'/' && bytes[pos + 1] == b'/' {
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
                continue;
            }

            // Field name
            let field_start = pos;
            while pos < bytes.len() && bytes[pos] != b'=' && bytes[pos] != b'}' && bytes[pos] != b',' {
                pos += 1;
            }
            if pos >= bytes.len() || bytes[pos] == b'}' {
                if pos < bytes.len() {
                    pos += 1;
                }
                break;
            }
            if bytes[pos] == b',' {
                pos += 1;
                continue;
            }

            let field_name = input[field_start..pos].trim().to_lowercase();
            pos += 1; // skip =

            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }

            // Field value
            let value = if pos < bytes.len() && bytes[pos] == b'{' {
                pos += 1;
                let mut depth = 1;
                let val_start = pos;
                while pos < bytes.len() && depth > 0 {
                    match bytes[pos] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        pos += 1;
                    }
                }
                let val = input[val_start..pos].trim().to_string();
                if pos < bytes.len() {
                    pos += 1;
                }
                val
            } else if pos < bytes.len() && bytes[pos] == b'"' {
                pos += 1;
                let val_start = pos;
                while pos < bytes.len() && bytes[pos] != b'"' {
                    pos += 1;
                }
                let val = input[val_start..pos].trim().to_string();
                if pos < bytes.len() {
                    pos += 1;
                }
                val
            } else {
                let val_start = pos;
                while pos < bytes.len() && bytes[pos] != b',' && bytes[pos] != b'}' && bytes[pos] != b'\n' {
                    pos += 1;
                }
                input[val_start..pos].trim().to_string()
            };

            let normalized = value.lines().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
            fields.insert(field_name, normalized);

            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos < bytes.len() && bytes[pos] == b',' {
                pos += 1;
            }
        }

        if !key.is_empty() {
            entries.insert(
                key.clone(),
                BibEntry {
                    key,
                    entry_type,
                    fields,
                },
            );
        }
    }

    return entries;
}

// ============================================================
// Formatting helpers
// ============================================================

/// Extract last name from an author string.
fn extract_last_name(author: &str) -> String {
    let clean = strip_bib_braces(author);
    if let Some(comma_pos) = clean.find(',') {
        clean[..comma_pos].trim().to_string()
    } else {
        clean.split_whitespace().last().unwrap_or("").to_string()
    }
}

/// Strip BibTeX braces: `{A {Unified} Theory}` → `A Unified Theory`.
fn strip_bib_braces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch != '{' && ch != '}' {
            result.push(ch);
        }
    }
    return result;
}

/// Format author names in AMS style with full first names.
/// "Lang, Serge" → "Serge Lang"
/// "Farber, Michael and Grant, Mark" → "Michael Farber and Mark Grant"
/// "Cohen, Daniel C. and Farber, Michael and Weinberger, Shmuel" → "Daniel C. Cohen, Michael Farber, and Shmuel
/// Weinberger"
fn format_authors_full(raw: &str) -> String {
    let authors: Vec<&str> = raw.split(" and ").collect();
    let formatted: Vec<String> = authors
        .iter()
        .map(|a| {
            let a = strip_bib_braces(a.trim());
            if let Some(comma_pos) = a.find(',') {
                let last = a[..comma_pos].trim();
                let first = a[comma_pos + 1..].trim();
                format!("{first} {last}")
            } else {
                a.to_string()
            }
        })
        .collect();

    match formatted.len() {
        0 => String::new(),
        1 => formatted[0].clone(),
        2 => format!("{} and {}", formatted[0], formatted[1]),
        _ => {
            let last = formatted.last().unwrap();
            let rest = &formatted[..formatted.len() - 1];
            format!("{}, and {last}", rest.join(", "))
        }
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_entry() {
        let input = r#"
@book{hatcher,
    author = {Allen Hatcher},
    title = {Algebraic topology},
    year = 2002,
    publisher = {Cambridge University Press},
}
"#;
        let entries = parse_bib_str(input);
        assert_eq!(entries.len(), 1);
        let entry = &entries["hatcher"];
        assert_eq!(entry.entry_type, "book");
        assert_eq!(entry.author(), "Allen Hatcher");
        assert_eq!(entry.title(), "Algebraic topology");
        assert_eq!(entry.year(), "2002");
    }

    #[test]
    fn test_citation_label() {
        let input = r#"
@book{lang_algebra,
    author = {Lang, Serge},
    title = {Algebra},
    year = {2002},
}
"#;
        let entries = parse_bib_str(input);
        let entry = &entries["lang_algebra"];
        assert_eq!(entry.short_author(), "Lang");
        assert_eq!(entry.citation_label(), "Lan02");
    }

    #[test]
    fn test_citation_label_two_authors() {
        let input = r#"
@book{dummit_foote,
    author = {Dummit, David S. and Foote, Richard M.},
    title = {Abstract Algebra},
    year = {2004},
}
"#;
        let entries = parse_bib_str(input);
        let entry = &entries["dummit_foote"];
        assert_eq!(entry.short_author(), "Dummit and Foote");
        assert_eq!(entry.citation_label(), "DF04");
    }

    #[test]
    fn test_citation_label_many_authors() {
        let input = r#"
@misc{param_tc,
    author = {Daniel C. Cohen and Michael Farber and Shmuel Weinberger},
    year = {2021},
}
"#;
        let entries = parse_bib_str(input);
        let entry = &entries["param_tc"];
        assert_eq!(entry.short_author(), "Cohen et al.");
        assert_eq!(entry.citation_label(), "CFW21");
    }

    #[test]
    fn test_format_authors_full() {
        assert_eq!(format_authors_full("Farber, Michael"), "Michael Farber");
        assert_eq!(
            format_authors_full("Farber, Michael and Grant, Mark"),
            "Michael Farber and Mark Grant"
        );
        assert_eq!(format_authors_full("Allen Hatcher"), "Allen Hatcher");
        assert_eq!(
            format_authors_full("Cohen, Daniel C. and Farber, Michael and Weinberger, Shmuel"),
            "Daniel C. Cohen, Michael Farber, and Shmuel Weinberger"
        );
    }

    #[test]
    fn test_strip_bib_braces() {
        assert_eq!(strip_bib_braces("{A {Unified} Theory}"), "A Unified Theory");
        assert_eq!(strip_bib_braces("plain text"), "plain text");
    }
}
