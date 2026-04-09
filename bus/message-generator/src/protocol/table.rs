use std::collections::HashMap;

use macro_utils::{get_crate_root, types::MESSAGE_BUS_SUBSCRIBER_CRATE};
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::protocol::{definitions::Definitions, primitive::PrimitiveType, types::ResolvedType};

macro_rules! insert_types {
    ($map:expr, { $($key:expr => $val:expr),* $(,)? }) => {
        $(
            $map.insert($key.to_string(), $val);
        )*
    };
}

#[derive(Debug)]
pub struct TypeTable {
    inner: HashMap<String, ResolvedType>,
}

impl Default for TypeTable {
    fn default() -> Self {
        let mut map = HashMap::new();
        insert_types!(map, {
            "u8" => ResolvedType::Primitive(PrimitiveType::U8),
            "u16" => ResolvedType::Primitive(PrimitiveType::U16),
            "u32" => ResolvedType::Primitive(PrimitiveType::U32),
            "u64" => ResolvedType::Primitive(PrimitiveType::U64),
            "u128" => ResolvedType::Primitive(PrimitiveType::U128),
            "i8" => ResolvedType::Primitive(PrimitiveType::I8),
            "i16" => ResolvedType::Primitive(PrimitiveType::I16),
            "i32" => ResolvedType::Primitive(PrimitiveType::I32),
            "i64" => ResolvedType::Primitive(PrimitiveType::I64),
            "i128" => ResolvedType::Primitive(PrimitiveType::I128),
            "f32" => ResolvedType::F32,
            "f64" => ResolvedType::F64,
            "char" => ResolvedType::Char,
            "bool" => ResolvedType::Bool,
            "String" => ResolvedType::String,
            "[remainder]" => ResolvedType::Remainder,
        });
        Self { inner: map }
    }
}

impl TypeTable {
    pub fn filter(&mut self) {
        self.inner.retain(|_, ty| match ty {
            ResolvedType::Char
            | ResolvedType::Bool
            | ResolvedType::F32
            | ResolvedType::F64
            | ResolvedType::Primitive(_)
            | ResolvedType::Variable(_)
            | ResolvedType::ByteArray(_)
            | ResolvedType::String
            | ResolvedType::Remainder => false,

            ResolvedType::Enum(_)
            | ResolvedType::Struct(_)
            | ResolvedType::Wrapper(_)
            | ResolvedType::ChunkCoder(_)
            | ResolvedType::Message(_) => true,
        });
    }

    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(name)
    }

    pub fn get_primitive(&self, ty: &str) -> Option<PrimitiveType> {
        let Some(resolved) = self.inner.get(ty) else {
            return None;
        };

        match resolved {
            ResolvedType::Primitive(prim) => Some(*prim),
            _ => None,
        }
    }

    pub fn get_type(&self, ty: &str) -> Option<&ResolvedType> {
        self.inner.get(ty)
    }
}

impl ToTokens for TypeTable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let types = self.inner.values();
        let crate_root = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
        tokens.extend(quote! {
            use #crate_root::{macros::Metadata, Destination, bytemuck, Mask32};
            use core::fmt::Display;
            use core::ops::{Deref, DerefMut};
            #(#types)*
        });
    }
}

pub fn resolve(mut definitions: Definitions) -> (Ident, TypeTable) {
    let mut table = TypeTable::default();
    let mut resolved_count = 0;

    while !definitions.definitions.is_empty() {
        definitions.definitions.retain(|def| {
            if !def.can_resolve(&table) {
                return true;
            }

            let resolved = def.clone().resolve(&table);
            resolved_count += 1;
            table.inner.insert(resolved.name(), resolved);
            false
        });

        if resolved_count == 0 {
            let reasons = definitions
                .definitions
                .iter()
                .map(|def| def.get_reason(&table).unwrap_err())
                .collect::<Vec<_>>();

            panic!("Error: {reasons:#?}");
        }
        resolved_count = 0;
    }

    (format_ident!("{}", definitions.protocol), table)
}
