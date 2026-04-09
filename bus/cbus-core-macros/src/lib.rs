#![allow(clippy::missing_panics_doc)]

mod define;

use macro_utils::get_crate_root;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Metadata, attributes(metadata))]
pub fn derive_metadata(input: TokenStream) -> TokenStream {
    let crate_root = get_crate_root("message-bus-subscriber");

    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let mut type_name: Option<syn::Expr> = None;
    let mut version: Option<syn::Expr> = None;

    if let Some(attr) = input.attrs.iter().find(|a| a.path().is_ident("metadata")) {
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let value = meta.value()?;
                type_name = Some(value.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("version") {
                let value = meta.value()?;
                version = Some(value.parse()?);
                return Ok(());
            }

            Err(meta.error("Unknown attribute"))
        });
    }

    if type_name.is_none() {
        return syn::Error::new_spanned(
            &name,
            r#"Metadata requires #[metadata(name = "type_name", version = 1)]"#,
        )
        .to_compile_error()
        .into();
    }

    let expanded = quote! {
        impl #impl_generics #crate_root::message::Metadata for #name #type_generics #where_clause {
            const MESSAGE_TYPE: &'static str = #type_name;
            const VERSION: u8 = #version;
        }

        impl #impl_generics #crate_root::message::MessageSuper for #name #type_generics #where_clause {
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Payload)]
pub fn derive_payload(input: TokenStream) -> TokenStream {
    let crate_root = get_crate_root("message-bus-subscriber");

    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl #crate_root::message::Payload for #name {}

        const _: () = {
            if core::mem::size_of::<#name>() != 28 { //TODO Use const
                panic!(concat!("Size of ", stringify!(#name), " must be exactly 28 bytes!."));
            }
        };
    };

    TokenStream::from(expanded)
}

///Available payload size: 28
#[proc_macro]
pub fn define_message(input: TokenStream) -> TokenStream {
    define::define_message(input)
}
