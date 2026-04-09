#![allow(non_snake_case)]
#![allow(clippy::inline_always)]

pub mod types;

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// # Panics
/// Паникует если указанная библиотека не найдена
#[must_use]
pub fn get_crate_root(orig_name: &str) -> TokenStream2 {
    let found_crate = crate_name(orig_name)
        .unwrap_or_else(|_| panic!("{orig_name} is not present in `Cargo.toml`"));

    match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!( #ident )
        }
    }
}
