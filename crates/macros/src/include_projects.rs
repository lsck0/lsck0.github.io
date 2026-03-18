use std::{env, fs, path::PathBuf};

use proc_macro::TokenStream;
use quote::quote;

use crate::toml_segments::{toml_segments_to_tokens, toml_string_value};

pub fn include_projects_impl(_input: TokenStream) -> TokenStream {
    let manifest_directory = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let toml_path = PathBuf::from(&manifest_directory).join("content/projects.toml");
    let file_content = fs::read_to_string(&toml_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {}", toml_path.display(), error));
    let parsed: toml::Value = file_content
        .parse()
        .unwrap_or_else(|error| panic!("invalid TOML in {}: {}", toml_path.display(), error));

    let entries = parsed
        .get("projects")
        .and_then(|value| value.as_array())
        .unwrap_or_else(|| panic!("expected [[projects]] in {}", toml_path.display()));

    let project_tokens: Vec<proc_macro2::TokenStream> = entries
        .iter()
        .map(|entry| {
            let table = entry.as_table().expect("each [[projects]] entry must be a table");
            let title = toml_segments_to_tokens(table.get("title").expect("project missing 'title'"));
            let description = toml_segments_to_tokens(table.get("description").expect("project missing 'description'"));
            let url = toml_string_value(table, "url");
            let status_string = toml_string_value(table, "status");
            let status = match status_string.as_str() {
                "maintained" => quote! { ProjectStatus::Maintained },
                "wip" | "work in progress" => quote! { ProjectStatus::WorkInProgress },
                "planned" => quote! { ProjectStatus::Planned },
                "abandoned" => quote! { ProjectStatus::Abandoned },
                other => panic!("unknown project status: {:?}", other),
            };
            quote! {
                ProjectEntry {
                    title: #title,
                    description: #description,
                    url: #url,
                    status: #status,
                }
            }
        })
        .collect();

    let output = quote! {
        &[#(#project_tokens),*]
    };
    return output.into();
}
