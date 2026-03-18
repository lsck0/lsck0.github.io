#![allow(clippy::needless_return)]

use proc_macro::TokenStream;

mod include_meta;
mod include_posts;
mod include_projects;
mod include_publications;
mod toml_segments;

#[proc_macro]
pub fn include_meta(input: TokenStream) -> TokenStream {
    return include_meta::include_meta_impl(input);
}

#[proc_macro]
pub fn include_posts(input: TokenStream) -> TokenStream {
    return include_posts::include_posts_impl(input);
}

#[proc_macro]
pub fn include_projects(input: TokenStream) -> TokenStream {
    return include_projects::include_projects_impl(input);
}

#[proc_macro]
pub fn include_publications(input: TokenStream) -> TokenStream {
    return include_publications::include_publications_impl(input);
}
