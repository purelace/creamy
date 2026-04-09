use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::protocol::types::TypeHandle;

#[derive(Debug, Clone)]
pub struct Wrapper {
    pub name: Ident,
    pub types: Vec<TypeHandle>,
    pub size: usize,
    pub align: usize,
}

impl Wrapper {
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub const fn get_size(&self) -> usize {
        self.size
    }

    pub const fn get_align(&self) -> usize {
        self.align
    }

    pub fn generate_code(&self) -> TokenStream {
        let name = &self.name;
        let types = &self.types;
        let deref_impl = self.generate_deref_impl();
        quote! {
            #[derive(Default, Debug, Copy, Clone, PartialEq)]
            pub struct #name(#(#types,)*);
            #deref_impl
        }
    }

    fn generate_deref_impl(&self) -> TokenStream {
        if self.types.len() != 1 {
            return TokenStream::default();
        }

        let name = &self.name;
        let ty = self.types.first().unwrap();

        quote! {
            impl Deref for #name {
                type Target = #ty;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        }
    }
}
