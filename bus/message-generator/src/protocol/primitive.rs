use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Ident, parse_quote};

use crate::protocol::types::TypeHandle;

#[derive(Debug, Clone, Copy)]
pub enum PrimitiveType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
}

impl ToTokens for PrimitiveType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token: Ident = match self {
            PrimitiveType::U8 => parse_quote!(u8),
            PrimitiveType::U16 => parse_quote!(u16),
            PrimitiveType::U32 => parse_quote!(u32),
            PrimitiveType::U64 => parse_quote!(u64),
            PrimitiveType::U128 => parse_quote!(u128),
            PrimitiveType::I8 => parse_quote!(i8),
            PrimitiveType::I16 => parse_quote!(i16),
            PrimitiveType::I32 => parse_quote!(i32),
            PrimitiveType::I64 => parse_quote!(i64),
            PrimitiveType::I128 => parse_quote!(i128),
        };

        tokens.extend(std::iter::once(token));
    }
}

impl From<&str> for PrimitiveType {
    fn from(value: &str) -> Self {
        match value {
            "u8" => PrimitiveType::U8,
            "u16" => PrimitiveType::U16,
            "u32" => PrimitiveType::U32,
            "u64" => PrimitiveType::U64,
            "u128" => PrimitiveType::U128,
            "i8" => PrimitiveType::I8,
            "i16" => PrimitiveType::I16,
            "i32" => PrimitiveType::I32,
            "i64" => PrimitiveType::I64,
            "i128" => PrimitiveType::I128,
            other => panic!("Unknown primitive type '{other}'"),
        }
    }
}

impl PrimitiveType {
    pub fn name(&self) -> String {
        match self {
            PrimitiveType::U8 => "u8",
            PrimitiveType::U16 => "u16",
            PrimitiveType::U32 => "u32",
            PrimitiveType::U64 => "u64",
            PrimitiveType::U128 => "u128",
            PrimitiveType::I8 => "i8",
            PrimitiveType::I16 => "i16",
            PrimitiveType::I32 => "i32",
            PrimitiveType::I64 => "i64",
            PrimitiveType::I128 => "i128",
        }
        .to_string()
    }
    pub const fn get_size(self) -> usize {
        match self {
            PrimitiveType::U8 | PrimitiveType::I8 => 1,
            PrimitiveType::U16 | PrimitiveType::I16 => 2,
            PrimitiveType::U32 | PrimitiveType::I32 => 4,
            PrimitiveType::U64 | PrimitiveType::I64 => 8,
            PrimitiveType::U128 | PrimitiveType::I128 => 16,
        }
    }

    pub const fn get_align(self) -> usize {
        self.get_size()
    }

    pub const fn get_handle(self) -> TypeHandle {
        match self {
            PrimitiveType::U8 => TypeHandle::U8,
            PrimitiveType::U16 => TypeHandle::U16,
            PrimitiveType::U32 => TypeHandle::U32,
            PrimitiveType::U64 => TypeHandle::U64,
            PrimitiveType::U128 => TypeHandle::U128,
            PrimitiveType::I8 => TypeHandle::I8,
            PrimitiveType::I16 => TypeHandle::I16,
            PrimitiveType::I32 => TypeHandle::I32,
            PrimitiveType::I64 => TypeHandle::I64,
            PrimitiveType::I128 => TypeHandle::I128,
        }
    }
}
