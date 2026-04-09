use quote::{ToTokens, quote};
use syn::{
    Expr, Ident, Token, braced, bracketed,
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

use crate::FieldDefinition;

pub struct TemplateFieldValueRef<'a> {
    pub name: &'a Ident,
    pub value: Option<&'a Expr>,
}

impl ToTokens for TemplateFieldValueRef<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        if let Some(value) = self.value {
            tokens.extend(quote! {
                #name: #value,
            });
        } else {
            tokens.extend(quote! {
                #name,
            });
        }
    }
}

impl<'a> TemplateFieldValueRef<'a> {
    pub const fn from_def(def: &'a FieldDefinition) -> Self {
        Self {
            name: &def.name,
            value: def.default.as_ref(),
        }
    }
}

pub struct TemplateFieldValue {
    pub name: Ident,
    pub value: Expr,
}

impl Parse for TemplateFieldValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let value = input.parse::<Expr>()?;
        Ok(Self { name, value })
    }
}

pub struct Template {
    pub name: Ident,
    pub values: Punctuated<TemplateFieldValue, Comma>,
}

impl Parse for Template {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;

        let content;
        parenthesized!(content in input);
        let values = content.parse_terminated(TemplateFieldValue::parse, Token![,])?;

        Ok(Self { name, values })
    }
}

enum TemplateOrValue {
    Template(Template),
    Value(TemplateFieldValue),
}

impl Parse for TemplateOrValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(Token![:]) {
            Ok(Self::Value(TemplateFieldValue::parse(input)?))
        } else if input.peek(Ident) && input.peek2(syn::token::Paren) {
            Ok(Self::Template(Template::parse(input)?))
        } else {
            Err(input.error("Unknown token"))
        }
    }
}

#[derive(Default)]
pub struct ScopedTemplates {
    pub globals: Vec<TemplateFieldValue>,
    pub templates: Vec<Template>,
}

impl Parse for ScopedTemplates {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut globals = Vec::new();
        let mut templates = Vec::new();

        while !input.is_empty() {
            match TemplateOrValue::parse(input)? {
                TemplateOrValue::Template(template) => templates.push(template),
                TemplateOrValue::Value(value) => globals.push(value),
            }
            if input.is_empty() {
                break;
            }

            input.parse::<syn::Token![,]>()?;
        }

        Ok(Self { globals, templates })
    }
}

pub struct Templates {
    pub inner: Vec<ScopedTemplates>,
}

impl Parse for Templates {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut global_scope = ScopedTemplates::default();
        let mut templates = Vec::new();

        while !input.is_empty() {
            if input.peek(syn::token::Brace) {
                let content;
                braced!(content in input);
                templates.push(ScopedTemplates::parse(&content)?);
                continue;
            }

            match TemplateOrValue::parse(input)? {
                TemplateOrValue::Template(template) => global_scope.templates.push(template),
                TemplateOrValue::Value(value) => global_scope.globals.push(value),
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        templates.push(global_scope);

        Ok(Self { inner: templates })
    }
}

pub fn parse_template_meta_field(input: ParseStream) -> syn::Result<Option<Templates>> {
    if !input.peek(Ident) {
        return Ok(None);
    }

    let name = Ident::parse_any(input)?;
    if name != "templates" {
        return Err(input.error("should be 'templates'"));
    }

    input.parse::<Token![:]>()?;

    let content;
    bracketed!(content in input);
    Ok(Some(Templates::parse(&content)?))
}
