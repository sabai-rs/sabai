//! Derive macros for Sabai. They emit metadata only and refer to runtime items through `::sabai::...`,
//! or whatever name the calling crate gives `sabai` or `sabai-core` in its Cargo.toml.

mod config_section;
mod model;
mod paths;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Ties a config struct to its file: `MailConfig` reads `config/mail.toml`.
/// Use `#[config("mail")]` when the struct name does not follow `<Section>Config`.
#[proc_macro_derive(ConfigSection, attributes(config))]
pub fn derive_config_section(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    config_section::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Maps a struct to a table, like an Eloquent model: `Post` reads the `posts` table.
/// Use `#[model(table = "people")]` when the plural is irregular.
#[proc_macro_derive(Model, attributes(model))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    model::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

pub(crate) fn snake_case(name: &str) -> String {
    let mut snake = String::with_capacity(name.len() + 4);
    for (index, character) in name.chars().enumerate() {
        if character.is_uppercase() && index > 0 {
            snake.push('_');
        }
        snake.extend(character.to_lowercase());
    }
    snake
}
