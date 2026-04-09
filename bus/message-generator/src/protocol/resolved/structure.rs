use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::protocol::resolved::field::Field;

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: Ident,
    pub fields: Vec<Field>,
    pub size: usize,
    pub align: usize,
}

impl Struct {
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub const fn get_size(&self) -> usize {
        todo!()
    }

    pub const fn get_align(&self) -> usize {
        todo!()
    }

    pub fn generate_code(&self) -> TokenStream {
        quote! {}
    }
}
