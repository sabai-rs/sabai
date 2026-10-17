use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, LitStr, Result};

use crate::{paths, snake_case};

pub(crate) fn expand(input: &DeriveInput) -> Result<TokenStream> {
    let orm = paths::orm()?;
    let name = &input.ident;
    let table = table_name(input)?;
    let fields = named_fields(input)?;
    let columns: Vec<String> = fields.iter().map(|field| field.to_string()).collect();
    Ok(quote! {
        impl #orm::Model for #name {
            const TABLE: &'static str = #table;
            const COLUMNS: &'static [&'static str] = &[#(#columns),*];

            fn from_row(row: &#orm::Row) -> #orm::Result<Self> {
                Ok(Self { #(#fields: row.get(#columns)?),* })
            }
        }
    })
}

fn named_fields(input: &DeriveInput) -> Result<Vec<&syn::Ident>> {
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            &input.ident,
            "`Model` needs a struct with named fields",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(Error::new_spanned(
            &input.ident,
            "`Model` needs a struct with named fields",
        ));
    };
    Ok(fields
        .named
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .collect())
}

fn table_name(input: &DeriveInput) -> Result<String> {
    let mut table = None;
    for attribute in input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("model"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                table = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            Err(meta.error("unknown `model` option; use `#[model(table = \"...\")]`"))
        })?;
    }
    Ok(table.unwrap_or_else(|| plural(&snake_case(&input.ident.to_string()))))
}

// Laravel's `Str::plural` for regular English; irregular ones use `#[model(table = "people")]`.
fn plural(word: &str) -> String {
    if let Some(stem) = word.strip_suffix('y')
        && !stem.ends_with(['a', 'e', 'i', 'o', 'u'])
    {
        return format!("{stem}ies");
    }
    if word.ends_with(['s', 'x', 'z']) || word.ends_with("ch") || word.ends_with("sh") {
        return format!("{word}es");
    }
    format!("{word}s")
}

#[cfg(test)]
mod tests {
    use super::plural;
    use crate::snake_case;

    #[test]
    fn struct_names_become_laravels_table_names() {
        let table = |name: &str| plural(&snake_case(name));

        assert_eq!(table("Post"), "posts");
        assert_eq!(table("BlogPost"), "blog_posts");
        assert_eq!(table("Category"), "categories");
        assert_eq!(table("Day"), "days");
        assert_eq!(table("Address"), "addresses");
        assert_eq!(table("Branch"), "branches");
    }
}
