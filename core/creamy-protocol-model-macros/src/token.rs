use proc_macro2::Span;
use quote::quote;
use syn::{
    DeriveInput, Expr, Field, Ident, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

struct ElementAttr {
    value: Expr,
    ty: Type,
}

impl Parse for ElementAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let value: Expr = input.parse()?;

        input.parse::<syn::Token![,]>()?;

        let ty: Type = input.parse()?;

        Ok(ElementAttr { value, ty })
    }
}

pub fn generate_vector_element_trait_impl(ast: &DeriveInput) -> proc_macro::TokenStream {
    let Some(attr) = ast.attrs.iter().find(|a| {
        if let Some(ident) = a.meta.path().get_ident() {
            ident == "element"
        } else {
            false
        }
    }) else {
        return proc_macro::TokenStream::default();
    };

    let syn::Meta::List(list) = &attr.meta else {
        return syn::Error::new(Span::call_site(), "Only list is allowed")
            .to_compile_error()
            .into();
    };

    let parsed_struct_attr = match list.parse_args::<ElementAttr>() {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error().into(), // Возвращаем compile_error, если синтаксис неверный
    };
    let name = &ast.ident;
    let max_size = parsed_struct_attr.value;
    let range_type = parsed_struct_attr.ty;

    quote! {
        impl crate::utils::VectorElement for #name {
            const MAX_SIZE: usize = #max_size;
            type RangeType = #range_type;
        }
    }
    .into()
}

pub fn generate_impl_block(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> proc_macro::TokenStream {
    let idents = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    let types = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();

    quote! {
        impl #name {
            pub const fn new(
                #(#idents: #types,)*
            ) -> Self {
                Self {
                    #(#idents,)*
                }
            }

            #(
                pub const fn #idents(&self) -> &#types {
                    &self.#idents
                }
            )*
        }

    }
    .into()
}
