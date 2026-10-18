use quote::{format_ident, quote};
use syn::{
    Attribute, DataEnum, Expr, Ident, ImplGenerics, LitInt, TypeGenerics, WhereClause,
    spanned::Spanned,
};

pub struct TokenContext<'a> {
    pub name: &'a Ident,
    pub impl_generics: ImplGenerics<'a>,
    pub ty_generics: TypeGenerics<'a>,
    pub where_clause: Option<&'a WhereClause>,
    pub magic: proc_macro2::TokenStream,
}

#[derive(Clone, Copy)]
pub enum Repr {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl Repr {
    pub fn to_ident(self) -> Ident {
        format_ident!("{}", self.to_string())
    }

    pub const fn to_string(self) -> &'static str {
        match self {
            Repr::I8 => "i8",
            Repr::I16 => "i16",
            Repr::I32 => "i32",
            Repr::I64 => "i64",
            Repr::U8 => "u8",
            Repr::U16 => "u16",
            Repr::U32 => "u32",
            Repr::U64 => "u64",
        }
    }

    pub fn find(attrs: &[Attribute]) -> Option<Self> {
        let mut detected_repr = None;

        for attr in attrs {
            if !attr.path().is_ident("repr") {
                continue;
            }

            let _ = attr.parse_nested_meta(|meta| {
                let path = &meta.path;
                let Some(ident) = path.get_ident() else {
                    return Ok(());
                };

                let repr = match ident.to_string().as_str() {
                    "i8" => Repr::I8,
                    "i16" => Repr::I16,
                    "i32" => Repr::I32,
                    "i64" => Repr::I64,
                    "u8" => Repr::U8,
                    "u16" => Repr::U16,
                    "u32" => Repr::U32,
                    "u64" => Repr::U64,
                    _ => return Ok(()),
                };

                detected_repr = Some(repr);
                Ok(())
            });
        }

        detected_repr
    }
}

fn combine_num_and_type(expr: &Expr, repr: Repr) -> proc_macro2::TokenStream {
    if let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(lit_int),
        ..
    }) = expr
    {
        let base_number = lit_int.base10_digits();

        let combined_string = format!("{}{}", base_number, repr.to_string());

        let new_lit_int = LitInt::new(&combined_string, lit_int.span());

        let token = Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit: syn::Lit::Int(new_lit_int),
        });

        quote! { #token }
    } else {
        syn::Error::new_spanned(
            expr,
            "Ожидалось простое числовое значение для привязки типа",
        )
        .to_compile_error()
    }
}

pub fn collect_read_variants(repr: Repr, s: &DataEnum) -> Vec<proc_macro2::TokenStream> {
    let mut vars = Vec::new();
    for variant in &s.variants {
        let Some((_, discriminant)) = &variant.discriminant else {
            return vec![
                syn::Error::new(variant.span(), "Missing explicit discriminant").to_compile_error(),
            ];
        };

        let ident = &variant.ident;
        let value = combine_num_and_type(discriminant, repr);

        let mut is_fields_unnamed = false;
        let fields = variant
            .fields
            .iter()
            .map(|f| {
                let ty = &f.ty;
                if let Some(ident) = f.ident.as_ref() {
                    quote! { #ident: #ty::read::<R, E>(reader)?, }
                } else {
                    is_fields_unnamed = true;
                    quote! { #ty::read::<R, E>(reader)?, }
                }
            })
            .collect::<Vec<_>>();

        let token = if fields.is_empty() {
            quote! { #value => Ok(Self::#ident), }
        } else if is_fields_unnamed {
            quote! {
                #value => Ok(Self::#ident (
                    #(#fields)*
                )),
            }
        } else {
            quote! {
                #value => Ok(Self::#ident {
                    #(#fields)*
                }),
            }
        };

        vars.push(token);
    }

    vars
}

pub fn collect_write_variants(repr: Repr, s: &DataEnum) -> Vec<proc_macro2::TokenStream> {
    let mut vars = Vec::new();
    for variant in &s.variants {
        let Some((_, discriminant)) = &variant.discriminant else {
            return vec![
                syn::Error::new(variant.span(), "Missing explicit discriminant").to_compile_error(),
            ];
        };

        let ident = &variant.ident;
        let value = combine_num_and_type(discriminant, repr);

        let mut is_fields_unnamed = false;
        let field_idents = variant
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                if let Some(ident) = f.ident.as_ref() {
                    quote! { #ident }
                } else {
                    is_fields_unnamed = true;
                    let ident = format_ident!("i{i}");
                    quote! { #ident }
                }
            })
            .collect::<Vec<_>>();

        let field_writes = variant
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let ident = f.ident.clone().unwrap_or(format_ident!("i{i}"));
                quote! { #ident.write::<W, E>(writer)?; }
            })
            .collect::<Vec<_>>();

        let token = if variant.fields.is_empty() {
            quote! { Self::#ident => #value.write::<W, E>(writer)?, }
        } else if is_fields_unnamed {
            quote! {
                Self::#ident(
                    #(#field_idents)*
                ) => {
                    #value.write::<W, E>(writer)?;
                    #(#field_writes)*
                }
            }
        } else {
            quote! {
                Self::#ident {
                    #(#field_idents)*
                } => {
                    #value.write::<W, E>(writer)?;
                    #(#field_writes)*
                }
            }
        };
        vars.push(token);
    }

    vars
}
