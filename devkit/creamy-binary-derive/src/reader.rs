use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, spanned::Spanned,
};

use crate::{
    attrs::magic::find_magic_bytes,
    utility::{Repr, TokenContext, collect_read_variants},
};

fn impl_reader_for_unnamed_fields(
    base: TokenContext,
    fields: &FieldsUnnamed,
) -> proc_macro::TokenStream {
    let names = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("f{i}"));

    let result = names.clone();

    let types = fields.unnamed.iter().map(|field| &field.ty);

    let TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    } = base;

    quote! {
        impl #impl_generics creamy_binary::BinaryReader for #name #ty_generics #where_clause {
            fn read<R: creamy_binary::Read, E: creamy_binary::Endianness>(reader: &mut R)
                -> Result<Self, creamy_binary::ReadError<R::Error>> {
                #magic
                #(
                    let #names = <#types>::read::<R, E>(reader)?;
                )*

                Ok(Self(
                    #(
                        #result,
                    )*
                ))
            }
        }
    }
    .into()
}

fn impl_reader_for_named_fields(
    base: TokenContext,
    fields: &FieldsNamed,
) -> proc_macro::TokenStream {
    let names = fields.named.iter().map(|field| field.ident.as_ref());
    let result = names.clone();
    let types = fields.named.iter().map(|field| &field.ty);

    let TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    } = base;

    quote! {
        impl #impl_generics creamy_binary::BinaryReader for #name #ty_generics #where_clause {
            fn read<R: creamy_binary::Read, E: creamy_binary::Endianness>(reader: &mut R)
                -> Result<Self, creamy_binary::ReadError<R::Error>> {
                #magic
                #(
                    let #names = <#types>::read::<R, E>(reader)?;
                )*

                Ok(Self {
                    #(
                        #result,
                    )*
                })
            }
        }
    }
    .into()
}

fn impl_reader_for_unit(base: TokenContext) -> proc_macro::TokenStream {
    let TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    } = base;

    quote! {
        impl #impl_generics creamy_binary::BinaryReader for #name #ty_generics #where_clause {
            fn read<R: creamy_binary::Read, E: creamy_binary::Endianness>(reader: &mut R)
                -> Result<Self, creamy_binary::ReadError<R::Error>> {
                #magic
                Ok(Self)
            }
        }
    }
    .into()
}

fn impl_reader_for_struct(base: TokenContext, s: &DataStruct) -> proc_macro::TokenStream {
    match &s.fields {
        Fields::Named(fields) => impl_reader_for_named_fields(base, fields),
        Fields::Unnamed(fields) => impl_reader_for_unnamed_fields(base, fields),
        Fields::Unit => impl_reader_for_unit(base),
    }
}

fn impl_reader_for_enum(base: TokenContext, repr: Repr, s: &DataEnum) -> proc_macro::TokenStream {
    let variants = collect_read_variants(repr, s);
    let TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    } = base;

    let ident = repr.to_ident();

    quote! {
        impl #impl_generics creamy_binary::BinaryReader for #name #ty_generics #where_clause {
            fn read<R: creamy_binary::Read, E: creamy_binary::Endianness>(reader: &mut R)
                -> Result<Self, creamy_binary::ReadError<R::Error>> {
                    #magic
                    let value = #ident::read::<R, E>(reader)?;
                    match value {
                        #(#variants)*
                        _ => Err(creamy_binary::ReadError::Custom(creamy_binary::CustomError {
                            message: format!("Unknown discriminant: {value}"),
                            error: None,
                        } ))
                    }
            }
        }
    }
    .into()
}

pub fn generate_reader_impl(ast: &DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let magic = if let Some(result) = find_magic_bytes(&ast.attrs) {
        match result {
            Ok(bytes) => {
                let len = bytes.len();
                quote! {
                    const MAGIC: [u8; #len] = [#(#bytes),*];
                    <[u8; #len]>::read::<R, E>(reader).and_then(|v| if v == MAGIC {
                        Ok(())
                    } else {
                        Err(creamy_binary::ReadError::Custom(creamy_binary::CustomError {
                            message: "Invalid magic byte".into(),
                            error: None,
                        }))
                    })?;
                }
            }

            Err(e) => return e.to_compile_error().into(),
        }
    } else {
        quote! {}
    };

    let base = TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    };

    match &ast.data {
        Data::Struct(s) => impl_reader_for_struct(base, s),
        Data::Enum(e) => {
            let Some(repr) = Repr::find(&ast.attrs) else {
                return syn::Error::new(ast.span(), "todo error")
                    .to_compile_error()
                    .into();
            };

            impl_reader_for_enum(base, repr, e)
        }
        Data::Union(_) => todo!(),
    }
}
