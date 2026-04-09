use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: Ident,
    pub variants: Vec<Ident>,
    pub size: usize,
}

impl Enum {
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub const fn get_size(&self) -> usize {
        self.size
    }

    pub const fn get_align(&self) -> usize {
        self.size
    }

    pub fn generate_code(&self) -> TokenStream {
        let name = &self.name;
        let variants = &self.variants;
        quote! {
            #[non_exhaustive]
            #[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum #name {
                #[default]
                #(
                    #variants,
                )*
            }

            impl Display for #name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #(
                            #name::#variants => write!(f, stringify!(#variants)),
                        )*
                    }
                }
            }
        }
    }
}
