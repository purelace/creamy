use proc_macro2::Span;
use quote::quote;
use syn::{
    DeriveInput, Expr,
    parse::{Parse, ParseStream},
};

struct KeyAttr {
    value: Expr,
}

impl Parse for KeyAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let value: Expr = input.parse()?;
        Ok(KeyAttr { value })
    }
}

pub fn generate_symbol_trait_impl(ast: &DeriveInput) -> proc_macro::TokenStream {
    let Some(attr) = ast.attrs.iter().find(|a| {
        if let Some(ident) = a.meta.path().get_ident() {
            ident == "key"
        } else {
            false
        }
    }) else {
        return syn::Error::new(Span::call_site(), "Missing `key` attribute")
            .to_compile_error()
            .into();
    };

    let syn::Meta::List(list) = &attr.meta else {
        return syn::Error::new(Span::call_site(), "Only list is allowed")
            .to_compile_error()
            .into();
    };

    let parsed_struct_attr = match list.parse_args::<KeyAttr>() {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error().into(),
    };
    let name = &ast.ident;
    let value = parsed_struct_attr.value;

    quote! {
        impl crate::model::storage::Symbol for #name {
            const KEY: crate::model::storage::SymbolKey = #value;
        }
    }
    .into()
}
