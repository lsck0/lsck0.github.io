use std::{env, fs, path::PathBuf};

use proc_macro::TokenStream;
use quote::quote;

pub fn include_meta_impl(_input: TokenStream) -> TokenStream {
    let manifest_directory = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let toml_path = PathBuf::from(&manifest_directory).join("content/meta.toml");
    let file_content = fs::read_to_string(&toml_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {}", toml_path.display(), error));
    let parsed: toml::Value = file_content
        .parse()
        .unwrap_or_else(|error| panic!("invalid TOML in {}: {}", toml_path.display(), error));

    // ---- Site-level metadata ----

    let site = parsed
        .get("site")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("expected [site] in {}", toml_path.display()));

    let site_title = str_field(site, "title");
    let site_description = str_field(site, "description");
    let site_author = str_field(site, "author");
    let site_url = str_field(site, "url");

    // ---- Per-page metadata ----

    let pages = parsed
        .get("pages")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("expected [pages] in {}", toml_path.display()));

    let page_tokens: Vec<proc_macro2::TokenStream> = pages
        .iter()
        .map(|(key, value)| {
            let table = value
                .as_table()
                .unwrap_or_else(|| panic!("pages.{key} must be a table"));
            let title = str_field(table, "title");
            let description = str_field(table, "description");
            quote! {
                (#key, PageMeta { title: #title, description: #description })
            }
        })
        .collect();

    let output = quote! {
        SiteMeta {
            title: #site_title,
            description: #site_description,
            author: #site_author,
            url: #site_url,
            pages: &[#(#page_tokens),*],
        }
    };
    return output.into();
}

fn str_field(table: &toml::value::Table, key: &str) -> String {
    return table.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string();
}
