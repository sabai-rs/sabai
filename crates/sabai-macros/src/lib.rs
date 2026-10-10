//! Derive macros for Sabai. They emit metadata only and refer to runtime items through `::sabai::...`,
//! or whatever name the calling crate gives `sabai` or `sabai-core` in its Cargo.toml.

mod config_section;
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
