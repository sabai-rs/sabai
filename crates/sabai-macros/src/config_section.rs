use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, LitStr, Result};

use crate::paths;

pub(crate) fn expand(input: &DeriveInput) -> Result<TokenStream> {
    let key = section_key(input)?;
    let core = paths::core()?;
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics #core::ConfigSection for #name #type_generics #where_clause {
            const KEY: &'static str = #key;
        }
    })
}

fn section_key(input: &DeriveInput) -> Result<String> {
    if let Some(attribute) = input
        .attrs
        .iter()
        .find(|attribute| attribute.path().is_ident("config"))
    {
        return explicit_key(attribute.parse_args()?);
    }
    let name = input.ident.to_string();
    match name.strip_suffix("Config") {
        Some(stem) if !stem.is_empty() => Ok(snake_case(stem)),
        _ => Err(Error::new_spanned(
            &input.ident,
            format!(
                "cannot tell which config file `{name}` reads; name it `<Section>Config` \
                 (`MailConfig` reads `config/mail.toml`) or add `#[config(\"mail\")]`"
            ),
        )),
    }
}

fn explicit_key(key: LitStr) -> Result<String> {
    let value = key.value();
    if value.is_empty() || value.contains(['/', '.']) {
        let hint = "use the file name without `.toml`, such as `#[config(\"mail\")]`";
        return Err(Error::new_spanned(
            key,
            format!("`{value}` is not a config section name; {hint}"),
        ));
    }
    Ok(value)
}

fn snake_case(name: &str) -> String {
    let mut snake = String::with_capacity(name.len() + 4);
    for (index, character) in name.chars().enumerate() {
        if character.is_uppercase() && index > 0 {
            snake.push('_');
        }
        snake.extend(character.to_lowercase());
    }
    snake
}

#[cfg(test)]
mod tests {
    use super::snake_case;

    #[test]
    fn struct_names_become_file_names() {
        assert_eq!(snake_case("Mail"), "mail");
        assert_eq!(snake_case("QueueWorker"), "queue_worker");
    }
}
