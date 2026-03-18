use quote::quote;

pub fn toml_segments_to_tokens(value: &toml::Value) -> proc_macro2::TokenStream {
    match value {
        toml::Value::String(text) => {
            quote! { &[TextSegment::Text(#text)] }
        }
        toml::Value::Array(elements) => {
            let segments: Vec<proc_macro2::TokenStream> = elements
                .iter()
                .map(|item| match item {
                    toml::Value::String(text) => quote! { TextSegment::Text(#text) },
                    toml::Value::Table(table) => {
                        if let Some(toml::Value::Integer(length)) = table.get("scrambled") {
                            let length = *length as usize;
                            quote! { TextSegment::Scrambled(#length) }
                        } else {
                            panic!("unknown table in text segment: {:?}", table);
                        }
                    }
                    _ => panic!("text segment must be string or table, got: {:?}", item),
                })
                .collect();
            quote! { &[#(#segments),*] }
        }
        _ => panic!("text field must be a string or array, got: {:?}", value),
    }
}

pub fn toml_string_value(table: &toml::value::Table, key: &str) -> String {
    return table
        .get(key)
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
}
