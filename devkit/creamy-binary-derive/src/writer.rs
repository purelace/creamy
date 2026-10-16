use quote::quote;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, FieldsNamed, FieldsUnnamed, Index, spanned::Spanned,
};

use crate::{
    attrs::magic::find_magic_bytes,
    utility::{Repr, TokenContext, collect_write_variants},
};

fn impl_writer_for_named_fields(
    ctx: TokenContext,
    fields: &FieldsNamed,
) -> proc_macro::TokenStream {
    let names = fields.named.iter().map(|field| field.ident.as_ref());
    let TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    } = ctx;

    quote! {
        impl #impl_generics creamy_binary::BinaryWriter for #name #ty_generics #where_clause {
            fn write<W: creamy_binary::Write<Error: Send + Sync + 'static>, E: creamy_binary::Endianness>(&self, writer: &mut W)
                -> Result<(), creamy_binary::WriteError<W::Error>> {
                #magic
                #(
                    self.#names.write::<W, E>(writer)?;
                )*

                Ok(())
            }
        }
    }
    .into()
}

fn impl_writer_for_unnamed_fields(
    base: TokenContext,
    fields: &FieldsUnnamed,
) -> proc_macro::TokenStream {
    let names = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, _)| Index::from(i));

    let TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    } = base;

    quote! {
        impl #impl_generics creamy_binary::BinaryWriter for #name #ty_generics #where_clause {
            fn write<W: creamy_binary::Write<Error: Send + Sync + 'static>, E: creamy_binary::Endianness>(&self, writer: &mut W)
                -> Result<(), creamy_binary::WriteError<W::Error>> {
                #magic
                #(
                    self.#names.write::<W, E>(writer)?;
                )*

                Ok(())
            }
        }
    }
    .into()
}

fn impl_writer_for_unit(base: TokenContext) -> proc_macro::TokenStream {
    let TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    } = base;

    quote! {
        impl #impl_generics creamy_binary::BinaryWriter for #name #ty_generics #where_clause {
            fn write<W: creamy_binary::Write<Error: Send + Sync + 'static>, E: creamy_binary::Endianness>(&self, writer: &mut W)
                -> Result<(), creamy_binary::WriteError<W::Error>> {
                #magic
                Ok(())
            }
        }
    }
    .into()
}

fn impl_writer_for_struct(base: TokenContext, s: &DataStruct) -> proc_macro::TokenStream {
    match &s.fields {
        syn::Fields::Named(f) => impl_writer_for_named_fields(base, f),
        syn::Fields::Unnamed(f) => impl_writer_for_unnamed_fields(base, f),
        syn::Fields::Unit => impl_writer_for_unit(base),
    }
}

fn impl_writer_for_enum(base: TokenContext, repr: Repr, s: &DataEnum) -> proc_macro::TokenStream {
    let variants = collect_write_variants(repr, s);
    let TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    } = base;

    quote! {
        impl #impl_generics creamy_binary::BinaryWriter for #name #ty_generics #where_clause {
            fn write<W: creamy_binary::Write<Error: Send + Sync + 'static>, E: creamy_binary::Endianness>(&self, writer: &mut W)
                -> Result<(), creamy_binary::WriteError<W::Error>> {
                    #magic
                    match self {
                        #(#variants)*
                    }

                    Ok(())
            }
        }
    }
    .into()
}

pub fn generate_writer_impl(ast: &DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let magic = if let Some(result) = find_magic_bytes(&ast.attrs) {
        match result {
            Ok(bytes) => {
                let len = bytes.len();
                quote! {
                    const MAGIC: [u8; #len] = [#(#bytes),*];
                    MAGIC.write::<W, E>(writer)?;
                }
            }
            Err(e) => return e.to_compile_error().into(),
        }
    } else {
        quote! {}
    };

    let ctx = TokenContext {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        magic,
    };

    match &ast.data {
        Data::Struct(s) => impl_writer_for_struct(ctx, s),
        Data::Enum(e) => {
            let Some(repr) = Repr::find(&ast.attrs) else {
                return syn::Error::new(ast.span(), "todo error")
                    .to_compile_error()
                    .into();
            };

            impl_writer_for_enum(ctx, repr, e)
        }
        Data::Union(_) => todo!(),
    }
}
