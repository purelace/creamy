use proc_macro2::Span;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, Field, Ident, Type,
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

struct TokenIdentAttr;

impl Parse for TokenIdentAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "ident" {
            Ok(Self)
        } else {
            Err(syn::Error::new(ident.span(), "Allowed only `ident` item"))
        }
    }
}

pub fn generate_with_ident_impl(ast: &DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(fields_named) => &fields_named.named,
            _ => {
                return syn::Error::new(
                    name.span(),
                    "Поддерживаются только структуры с именованными полями",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new(
                name.span(),
                "Макрос можно использовать только со структурами",
            )
            .to_compile_error()
            .into();
        }
    };

    let Some(ident_field) = fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|id| id == "ident"))
    else {
        return syn::Error::new(name.span(), "Missing `ident` field")
            .to_compile_error()
            .into();
    };

    let Some(attr) = ident_field.attrs.iter().find(|a| {
        if let Some(ident) = a.meta.path().get_ident() {
            ident == "token"
        } else {
            false
        }
    }) else {
        return syn::Error::new(Span::call_site(), "Missing `token` attribute")
            .to_compile_error()
            .into();
    };

    let syn::Meta::List(list) = &attr.meta else {
        return syn::Error::new(Span::call_site(), "Only list is allowed")
            .to_compile_error()
            .into();
    };

    if let Err(e) = list.parse_args::<TokenIdentAttr>() {
        return e.to_compile_error().into();
    }

    quote! {
        impl #name {
            #[must_use]
            pub const fn with_ident(&self, value: creamy_utils::strpool::StringId) -> Self {
                let mut copy = *self;
                copy.ident = value;
                copy
            }
        }
    }
    .into()
}
//impl $name {
//    #[allow(unused)]
//    pub const fn new($($field: $field_type,)*) -> Self {
//        Self { $($field,)* }
//    }
//
//    $(
//        #[allow(clippy::len_without_is_empty)]
//        pub const fn $field(&self) -> $field_type {
//            self.$field
//        }
//    )*
//}

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
        //#[allow(unused)]
        impl #name {
            pub const fn new(
                #(#idents: #types,)*
            ) -> Self {
                Self {
                    #(#idents,)*
                }
            }

            #(
                pub const fn #idents(&self) -> #types {
                    self.#idents
                }
            )*
        }

    }
    .into()
}
