#![allow(non_snake_case)]
#![allow(clippy::missing_errors_doc)]

mod field;
pub mod protocol;
mod templates;

pub use field::*;
use indexmap::IndexMap;
use macro_utils::{
    get_crate_root,
    types::{
        Bytemuck, Destination, MESSAGE_BUS_SUBSCRIBER_CRATE, MessageType, Metadata, MetadataDerive,
        TypedMessage,
    },
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Expr, Ident, Token, Type, braced,
    ext::IdentExt,
    parse::{Parse, ParseStream, Parser},
    parse_quote,
    punctuated::Punctuated,
};

use crate::templates::{TemplateFieldValueRef, Templates, parse_template_meta_field};

#[derive(Clone)]
pub struct GeneratedFields<'a> {
    slice: &'a [&'a FieldDefinition],
}

impl GeneratedFields<'_> {
    #[must_use]
    pub fn ctor_fields(&self) -> Vec<&FieldDefinition> {
        self.slice.iter().filter(|f| f.ctor).copied().collect()
    }

    #[must_use]
    pub fn no_ctor_fields(&self) -> Vec<&FieldDefinition> {
        self.slice.iter().filter(|f| !f.ctor).copied().collect()
    }

    #[must_use]
    pub fn all_field_types(&self) -> Vec<&Type> {
        self.slice.iter().map(|f| &f.ty).collect()
    }
}

pub enum MetaField<'a> {
    Define { name: &'a str },
    Override { name: &'a str, default: Expr },
}

pub trait CodeGenerator: 'static {
    fn meta_fields(&self) -> Vec<MetaField<'_>>;
    fn accept_meta_field(&mut self, name: &str, value: Expr);
    fn fields(&self) -> Vec<FieldDefinition>;
    fn generate_code(&self, name: &Ident, generated: GeneratedFields) -> proc_macro2::TokenStream;
    fn validate(&self);
}

fn should_be(ident: &Ident, name: &str) {
    assert_eq!(ident.to_string(), name, "Must be '{name}'");
}

pub struct MessageGenerator {
    providers: Vec<Box<dyn CodeGenerator>>,
    meta_fields: IndexMap<String, (usize, Option<Expr>)>, //Name - Generator index, Default Value
    builtin_fields: IndexMap<String, FieldDefinition>,
    user_fields: syn::punctuated::Punctuated<FieldDefinition, Token![,]>,
}

impl Default for MessageGenerator {
    fn default() -> Self {
        let Destination = Destination();
        let MessageType = MessageType();

        let mut builtin_fields = IndexMap::new();
        builtin_fields.insert(
            "dst".to_string(),
            FieldDefinition {
                ctor: true,
                name: parse_quote!(dst),
                ty: parse_quote!(#Destination),
                default: None,
            },
        );

        builtin_fields.insert(
            "src".to_string(),
            FieldDefinition {
                ctor: true,
                name: parse_quote!(src),
                ty: parse_quote!(u8),
                default: None,
            },
        );

        builtin_fields.insert(
            "group".to_string(),
            FieldDefinition {
                ctor: true,
                name: parse_quote!(group),
                ty: parse_quote!(u8),
                default: None,
            },
        );

        builtin_fields.insert(
            "kind".to_string(),
            FieldDefinition {
                ctor: true,
                name: parse_quote!(kind),
                ty: parse_quote!(u8),
                default: None,
            },
        );

        builtin_fields.insert(
            "version".to_string(),
            FieldDefinition {
                ctor: false,
                name: parse_quote!(version),
                ty: parse_quote!(u8),
                default: Some(parse_quote!(Self::VERSION)),
            },
        );

        let mut meta_fields = IndexMap::new();
        meta_fields.insert("version".to_string(), (usize::MAX, None));
        meta_fields.insert("type".to_string(), (usize::MAX, None));

        Self {
            providers: Vec::new(),
            meta_fields,
            builtin_fields,
            user_fields: Punctuated::default(),
        }
    }
}

impl MessageGenerator {
    pub fn add_generator(&mut self, generator: impl CodeGenerator) {
        self.providers.push(Box::new(generator));
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn prepare_data(&mut self) -> syn::Result<()> {
        for (index, provider) in self.providers.iter().enumerate() {
            for meta_field in provider.meta_fields() {
                match meta_field {
                    MetaField::Define { name } => {
                        if self
                            .meta_fields
                            .insert(name.to_string(), (index, None))
                            .is_some()
                        {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                format!("Meta field with name '{name}' was already added"),
                            ));
                        }
                    }
                    MetaField::Override { name, default } => {
                        let index = self.meta_fields.get_index_of(name);
                        let Some(target_index) = index else {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                format!(
                                    "Cannot override: meta field with name '{name}' does not exist"
                                ),
                            ));
                        };

                        let (index, _) = self.meta_fields.swap_remove(name).unwrap();
                        self.meta_fields
                            .insert(name.to_string(), (index, Some(default)));
                        let last_index = self.meta_fields.len().saturating_sub(1);

                        self.meta_fields.swap_indices(last_index, target_index);
                    }
                }
            }

            for field_def in provider.fields() {
                let field_name = field_def.name.to_string();
                if self
                    .builtin_fields
                    .insert(field_def.name.to_string(), field_def)
                    .is_some()
                {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!("Struct field with name '{field_name}' was already added"),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate(&self) {
        for provider in &self.providers {
            provider.validate();
        }
    }

    fn generate_code(&self, general: &mut GeneralData) -> TokenStream2 {
        let gen_struct = self.gen_struct(general);
        let gen_typed_impl = Self::gen_typed_impl(general);
        let gen_struct_impl = self.gen_struct_impl(general);
        let gen_template = self.gen_templates(general);

        let mut generated: Vec<&FieldDefinition> =
            self.builtin_fields.iter().map(|f| f.1).collect();
        generated.extend(self.user_fields.iter());

        let generated = GeneratedFields {
            slice: generated.as_slice(),
        };

        let custom = self
            .providers
            .iter()
            .map(|p| p.generate_code(&general.name, generated.clone()));

        quote! {
            #gen_struct
            #gen_typed_impl
            #gen_struct_impl
            #gen_template
            #(
                #custom
            )*
        }
    }

    fn gen_struct(&self, general: &GeneralData) -> proc_macro2::TokenStream {
        let type_name = &general.type_name;
        let version = &general.version;
        let name = &general.name;
        let MetadataDerive = MetadataDerive();
        let Bytemuck = Bytemuck();

        let field_names: Vec<&Ident> = self
            .builtin_fields
            .values()
            .chain(self.user_fields.iter())
            .map(|f| &f.name)
            .collect();

        let field_types: Vec<&Type> = self
            .builtin_fields
            .values()
            .chain(self.user_fields.iter())
            .map(|f| &f.ty)
            .collect();

        quote! {
            #[repr(C, align(32))]
            #[derive(#MetadataDerive, Default, Debug, Copy, Clone)]
            #[metadata(name = #type_name, version = #version)]
            pub struct #name {
                #(
                    pub #field_names: #field_types,
                )*
                pub padding: [u8; Self::PADDING_SIZE]
            }

            unsafe impl #Bytemuck::Zeroable for #name {}
            unsafe impl #Bytemuck::Pod for #name {}
        }
    }

    fn gen_typed_impl(general: &GeneralData) -> proc_macro2::TokenStream {
        let crate_name = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
        let name = &general.name;
        let TypedMessage = TypedMessage();
        let Destination = Destination();
        let MessageType = MessageType();
        let Bytemuck = Bytemuck();

        quote! {
            impl #TypedMessage for #name {
                #[inline(always)]
                fn dst(&self) -> #Destination {
                    self.dst
                }

                #[inline(always)]
                fn set_dst(&mut self, dst: #Destination) {
                    self.dst = dst;
                }

                #[inline(always)]
                fn src(&self) -> u8 {
                    self.src
                }

                #[inline(always)]
                fn version(&self) -> u8 {
                    self.version
                }

                #[inline(always)]
                fn set_version(&mut self, version: u8) {
                    self.version = version;
                }

                #[inline(always)]
                fn kind(&self) -> #MessageType {
                    self.kind
                }

                #[inline(always)]
                fn set_kind(&mut self, ty: #MessageType) {
                    self.kind = ty;
                }

                #[inline(always)]
                fn as_raw_bytes(&self) -> &[u8; #crate_name::MESSAGE_SIZE] {
                    #Bytemuck::cast_ref(self)
                }

                #[inline(always)]
                fn payload_as_raw_bytes(&self) -> &[u8; #crate_name::PAYLOAD_SIZE] {
                    let full_data = self.as_raw_bytes();
                    unsafe {
                        &*full_data.as_ptr().add(4).cast::<[u8; 28]>()
                    }
                }
            }
        }
    }

    fn gen_struct_impl(&self, general: &GeneralData) -> proc_macro2::TokenStream {
        let crate_name = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
        let name = &general.name;
        let Metadata = Metadata();

        let ctor_fields: Vec<&FieldDefinition> = self
            .builtin_fields
            .values()
            .chain(self.user_fields.iter())
            .filter(|f| f.ctor)
            .collect();

        let ctor_field_names: Vec<&Ident> = ctor_fields.iter().map(|f| &f.name).collect();
        let ctor_field_types: Vec<&Type> = ctor_fields.iter().map(|f| &f.ty).collect();

        let no_ctor_field_names: Vec<&Ident> = self
            .builtin_fields
            .values()
            .chain(self.user_fields.iter())
            .filter(|f| !f.ctor)
            .map(|f| &f.name)
            .collect();

        let no_ctor_default_values: Vec<&Expr> = self
            .builtin_fields
            .values()
            .chain(self.user_fields.iter())
            .filter(|f| !f.ctor)
            .map(|f| {
                f.default
                    .as_ref()
                    .expect("No constructor fields should have const eval default value")
            })
            .collect();

        let field_types: Vec<&Type> = self
            .builtin_fields
            .values()
            .chain(self.user_fields.iter())
            .map(|f| &f.ty)
            .collect();

        quote! {
            impl #name {
                const ___ASSERT_SIZE___: () = assert!(core::mem::size_of::<#name>() == #crate_name::MESSAGE_SIZE);

                pub const PADDING_SIZE: usize =
                    #crate_name::MESSAGE_SIZE - (
                        core::mem::size_of::<(
                            #(
                                #field_types,
                            )*
                        )>()
                    );

                #[must_use]
                #[inline(always)]
                #[allow(clippy::too_many_arguments)]
                pub const fn new(
                    #(
                        #ctor_field_names: #ctor_field_types,
                    )*
                ) -> Self {
                    use #Metadata;
                    Self {
                        #(
                            #ctor_field_names,
                        )*
                        #(
                            #no_ctor_field_names: #no_ctor_default_values,
                        )*
                        padding: [0; Self::PADDING_SIZE],
                    }
                }

                #[must_use]
                #[inline(always)]
                pub const fn as_u128(&self) -> u128 {
                    unsafe {
                        let array = std::mem::transmute::<&Self, &[u8; 32]>(self);
                        let first_half = (&array).as_ptr().cast::<[u8; 16]>();
                        *first_half.cast::<u128>()
                    }
                }
            }
        }
    }

    fn gen_templates(&self, general: &mut GeneralData) -> TokenStream2 {
        let Some(templates) = general.templates.take() else {
            return quote! {};
        };
        let Metadata = Metadata();

        /*
         *  Берем все поля
         *  Устанавливаем значения
         *  Добавляем в аргументы только те что не имеют значения и без [no_ctor]
         */

        let mut generated = Vec::new();

        for scoped in templates.inner {
            let fields = self
                .builtin_fields
                .values()
                .chain(self.user_fields.iter())
                .map(|f| (f.name.to_string(), f.clone()));
            let mut field_map = IndexMap::new();
            field_map.extend(fields);

            for value in scoped.globals {
                let name = value.name.to_string();
                let def = field_map.get_mut(&name).unwrap();
                def.default = Some(value.value);
            }

            for template in scoped.templates {
                let mut field_map = field_map.clone();

                let name = template.name;
                for value in template.values {
                    let name = value.name.to_string();
                    let def = field_map.get_mut(&name).unwrap();
                    def.default = Some(value.value);
                }

                let init_iter = field_map.values().map(TemplateFieldValueRef::from_def);

                let arg_iter = field_map
                    .values()
                    .filter(|f| f.ctor && f.default.is_none())
                    .map(FieldDefinitionRef::from_def);

                generated.push(quote! {
                    #[must_use]
                    #[inline(always)]
                    pub const fn #name(
                        #(#arg_iter)*
                    ) -> Self {
                        use #Metadata;
                        Self {
                            padding: [0; Self::PADDING_SIZE],
                            #(#init_iter)*
                        }
                    }
                });
            }
        }

        let name = &general.name;
        quote! {
            impl #name {
                #(#generated)*
            }
        }
    }
}

/// name: value,
fn parse_meta_field_expr(input: ParseStream, name: &str) -> syn::Result<Expr> {
    should_be(&Ident::parse_any(input)?, name);
    input.parse::<Token![:]>()?;
    let expr = input.parse::<Expr>()?;
    input.parse::<Token![,]>()?;

    Ok(expr)
}

pub struct GeneralData {
    version: Expr,
    type_name: Expr,
    name: Ident,
    templates: Option<Templates>,
}

pub fn generate_struct(
    mut generator: MessageGenerator,
    input: TokenStream2,
) -> syn::Result<TokenStream2> {
    generator.prepare_data()?;

    let parser = |input: ParseStream| -> syn::Result<GeneralData> {
        let version = generator
            .meta_fields
            .drain(..1)
            .map(|(_, (_, default))| default)
            .next()
            .flatten()
            .map_or_else(|| parse_meta_field_expr(input, "version"), Ok)?;

        let type_name = generator
            .meta_fields
            .drain(..1)
            .map(|(_, (_, default))| default)
            .next()
            .flatten()
            .map_or_else(|| parse_meta_field_expr(input, "type"), Ok)?;

        let overriden = parse_override_meta_field(input)?;

        if let Some(overriden) = overriden {
            for overriden_field in overriden.list {
                let name = overriden_field.name.to_string();
                let def = generator
                    .builtin_fields
                    .get_mut(&name)
                    .expect("Target field does not exist");
                def.ctor = overriden_field.ctor;
                def.default = Some(overriden_field.value);
            }
        }

        for (name, (index, default_value)) in generator.meta_fields.drain(..) {
            let value = if let Some(expr) = default_value {
                expr
            } else {
                parse_meta_field_expr(input, &name)?
            };

            let provider = &mut generator.providers[index];
            provider.accept_meta_field(&name, value);
        }

        let name = input.parse::<Ident>()?;

        let content;
        braced!(content in input);
        let fields = content.parse_terminated(FieldDefinition::parse, Token![,])?;

        let templates = parse_template_meta_field(input)?;

        generator.user_fields.extend(fields);
        Ok(GeneralData {
            version,
            type_name,
            name,
            templates,
        })
    };

    let mut general = parser.parse2(input)?;
    generator.validate();
    Ok(generator.generate_code(&mut general))
}

//TODO
// override builtin fields
// template function generation
// tests

/*
 * Parse AST
 * Parse extensions AST
 * Resolve types
 * Resolve extensions
 * Build final types
 * Apply extensions
 * Generate code
 */
