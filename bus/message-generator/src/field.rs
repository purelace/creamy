use quote::{ToTokens, quote};
use syn::{
    Expr, Ident, Token, Type, bracketed,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

#[derive(Clone)]
pub struct FieldDefinition {
    pub ctor: bool,
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
}

fn read_ctor_attr(input: ParseStream) -> syn::Result<bool> {
    if input.peek(syn::token::Bracket) {
        let content;
        syn::bracketed!(content in input);

        if content.peek(syn::Ident) {
            let id: syn::Ident = content.parse()?;
            if id == "no_ctor" {
                Ok(false)
            } else {
                panic!("should be 'no_ctor'");
            }
        } else {
            panic!("should be 'no_ctor'");
        }
    } else {
        Ok(true)
    }
}

impl Parse for FieldDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // [no_ctor]
        let ctor = read_ctor_attr(input)?;

        // name: type
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;

        // name: type = value
        let default = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse::<Expr>()?)
        } else {
            None
        };

        Ok(Self {
            ctor,
            name,
            ty,
            default,
        })
    }
}

/// Syntax:
/// `[CTOR_ATTR]` `Ident`: `Expr`,
///
/// # Examples:
/// * name: "John"
/// * \[`no_ctor`] id: 1000,
pub struct OverridenField {
    pub ctor: bool,
    pub name: Ident,
    pub value: Expr,
}

impl Parse for OverridenField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ctor = read_ctor_attr(input)?;

        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let value = input.parse::<Expr>()?;

        Ok(Self { ctor, name, value })
    }
}

pub struct OverridenFields {
    pub list: Punctuated<OverridenField, Comma>,
}

pub fn parse_override_meta_field(input: ParseStream) -> syn::Result<Option<OverridenFields>> {
    if !input.peek(Token![override]) {
        return Ok(None);
    }

    Ident::parse_any(input)?;
    input.parse::<Token![:]>()?;

    let content;
    bracketed!(content in input);
    let fields = content.parse_terminated(OverridenField::parse, Token![,])?;

    Ok(Some(OverridenFields { list: fields }))
}

pub struct FieldDefinitionRef<'a> {
    pub name: &'a Ident,
    pub ty: &'a Type,
}

impl<'a> FieldDefinitionRef<'a> {
    #[must_use]
    pub const fn from_def(def: &'a FieldDefinition) -> Self {
        Self {
            name: &def.name,
            ty: &def.ty,
        }
    }
}

impl ToTokens for FieldDefinitionRef<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        let ty = self.ty;
        tokens.extend(quote! {
            #name: #ty,
        });
    }
}
