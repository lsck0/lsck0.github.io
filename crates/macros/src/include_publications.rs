use std::{env, fs, path::PathBuf};

use proc_macro::TokenStream;
use quote::quote;

use crate::toml_segments::{toml_segments_to_tokens, toml_string_value};

pub fn include_publications_impl(_input: TokenStream) -> TokenStream {
    let manifest_directory = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let toml_path = PathBuf::from(&manifest_directory).join("content/publications.toml");
    let file_content = fs::read_to_string(&toml_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {}", toml_path.display(), error));
    let parsed: toml::Value = file_content
        .parse()
        .unwrap_or_else(|error| panic!("invalid TOML in {}: {}", toml_path.display(), error));

    let entries = parsed
        .get("publications")
        .and_then(|value| value.as_array())
        .unwrap_or_else(|| panic!("expected [[publications]] in {}", toml_path.display()));

    let publication_tokens: Vec<proc_macro2::TokenStream> = entries
        .iter()
        .map(|entry| {
            let table = entry.as_table().expect("each [[publications]] entry must be a table");
            let title = toml_segments_to_tokens(table.get("title").expect("publication missing 'title'"));
            let description =
                toml_segments_to_tokens(table.get("description").expect("publication missing 'description'"));
            let url = toml_string_value(table, "url");
            let authors = toml_segments_to_tokens(table.get("authors").unwrap_or(&toml::Value::String(String::new())));
            let date = toml_segments_to_tokens(table.get("date").unwrap_or(&toml::Value::String(String::new())));
            quote! {
                PublicationEntry {
                    title: #title,
                    description: #description,
                    url: #url,
                    authors: #authors,
                    date: #date,
                }
            }
        })
        .collect();

    let output = quote! {
        &[#(#publication_tokens),*]
    };
    return output.into();
}
