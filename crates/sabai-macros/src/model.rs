use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, LitStr, Result};

use crate::{paths, snake_case};

pub(crate) fn expand(input: &DeriveInput) -> Result<TokenStream> {
    let orm = paths::orm()?;
    let name = &input.ident;
    let options = options(input)?;
    let table = options
        .table
        .unwrap_or_else(|| plural(&snake_case(&name.to_string())));
    let connection = options.connection.map(|connection| {
        quote!(const CONNECTION: ::core::option::Option<&'static str> = ::core::option::Option::Some(#connection);)
    });
    let fields = named_fields(input)?;
    let columns: Vec<String> = fields.iter().map(|field| field.to_string()).collect();
    if !columns.iter().any(|column| column == "id") {
        return Err(Error::new_spanned(
            name,
            "`Model` needs an `id: u64` field for the primary key, the column `table.id()` makes in a migration",
        ));
    }
    let has = |field: &str| columns.iter().any(|column| column == field);
    let timestamps = (has("created_at") && has("updated_at")).then(|| {
        quote! {
            fn timestamps(&mut self) -> ::core::option::Option<#orm::Timestamps<'_>> {
                ::core::option::Option::Some(#orm::Timestamps {
                    created_at: &mut self.created_at,
                    updated_at: &mut self.updated_at,
                })
            }
        }
    });
    Ok(quote! {
        impl #orm::Model for #name {
            const TABLE: &'static str = #table;
            const COLUMNS: &'static [&'static str] = &[#(#columns),*];
            #connection

            fn from_row(row: &#orm::Row) -> #orm::Result<Self> {
                Ok(Self { #(#fields: row.get(#columns)?),* })
            }

            fn values(&self) -> #orm::Result<::std::vec::Vec<#orm::SqlValue>> {
                Ok(::std::vec![#(#orm::ToColumn::to_column(&self.#fields)?),*])
            }

            fn id(&self) -> u64 {
                self.id
            }

            fn set_id(&mut self, id: u64) {
                self.id = id;
            }

            #timestamps
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

#[derive(Default)]
struct Options {
    table: Option<String>,
    connection: Option<String>,
}

fn options(input: &DeriveInput) -> Result<Options> {
    let mut options = Options::default();
    for attribute in input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("model"))
    {
        attribute.parse_nested_meta(|meta| {
            let value = meta.value()?.parse::<LitStr>()?.value();
            if meta.path.is_ident("table") {
                options.table = Some(value);
            } else if meta.path.is_ident("connection") {
                options.connection = Some(value);
            } else {
                return Err(meta.error(
                    "unknown `model` option; use `table = \"...\"` or `connection = \"...\"`",
                ));
            }
            Ok(())
        })?;
    }
    Ok(options)
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
    fn struct_names_become_plural_snake_case_table_names() {
        let table = |name: &str| plural(&snake_case(name));

        assert_eq!(table("Post"), "posts");
        assert_eq!(table("BlogPost"), "blog_posts");
        assert_eq!(table("Category"), "categories");
        assert_eq!(table("Day"), "days");
        assert_eq!(table("Address"), "addresses");
        assert_eq!(table("Branch"), "branches");
    }
}
