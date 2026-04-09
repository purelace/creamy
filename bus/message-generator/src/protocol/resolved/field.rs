use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Ident;

use crate::protocol::types::TypeHandle;

#[derive(Debug, Clone)]
pub struct Field {
    pub name: Ident,
    pub ty: TypeHandle,
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let ty = &self.ty;
        let token = quote!(#name: #ty);
        tokens.extend(std::iter::once(token));
    }
}
