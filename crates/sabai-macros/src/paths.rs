use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Error, Result};

/// The path to `sabai-core` items as the calling crate sees them: `::sabai`, a renamed
/// `sabai` (`::web`), `::sabai_core`, or `crate` inside `sabai-core` itself.
pub(crate) fn core() -> Result<TokenStream> {
    // `Itself` here means the facade's integration tests, where `crate` is the test crate.
    match crate_name("sabai") {
        Ok(FoundCrate::Itself) => return Ok(quote!(::sabai)),
        Ok(FoundCrate::Name(name)) => return Ok(absolute(&name)),
        Err(_) => {}
    }
    match crate_name("sabai-core") {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => Ok(absolute(&name)),
        Err(_) => Err(Error::new(
            Span::call_site(),
            "this derive needs the `sabai` crate; add `sabai` to `[dependencies]` in Cargo.toml",
        )),
    }
}

/// The path to `sabai-orm` items: `::sabai::orm`, or `::sabai_orm` and `crate` without the facade.
pub(crate) fn orm() -> Result<TokenStream> {
    match crate_name("sabai") {
        Ok(FoundCrate::Itself) => return Ok(quote!(::sabai::orm)),
        Ok(FoundCrate::Name(name)) => {
            let name = absolute(&name);
            return Ok(quote!(#name::orm));
        }
        Err(_) => {}
    }
    match crate_name("sabai-orm") {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => Ok(absolute(&name)),
        Err(_) => Err(Error::new(
            Span::call_site(),
            "`Model` needs `sabai` with the `orm` feature; add `features = [\"orm\"]` to `sabai` in Cargo.toml",
        )),
    }
}

fn absolute(name: &str) -> TokenStream {
    let name = format_ident!("{name}");
    quote!(::#name)
}
