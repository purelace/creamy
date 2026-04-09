use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident};
use syn::parse_quote;

use crate::protocol::{
    primitive::PrimitiveType,
    resolved::{ChunkCoder, Enum, Message, Struct, Wrapper},
};

#[derive(Debug, Clone)]
pub enum TypeHandle {
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
    F32,
    F64,
    Char,
    Bool,
    String,
    ByteArray(usize),
    Other(String),
}

impl ToTokens for TypeHandle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token: TokenStream = match self {
            TypeHandle::U8 => parse_quote!(u8),
            TypeHandle::U16 => parse_quote!(u16),
            TypeHandle::U32 => parse_quote!(u32),
            TypeHandle::U64 => parse_quote!(u64),
            TypeHandle::U128 => parse_quote!(u128),
            TypeHandle::I8 => parse_quote!(i8),
            TypeHandle::I16 => parse_quote!(i16),
            TypeHandle::I32 => parse_quote!(i32),
            TypeHandle::I64 => parse_quote!(i64),
            TypeHandle::I128 => parse_quote!(i128),
            TypeHandle::F32 => parse_quote!(f32),
            TypeHandle::F64 => parse_quote!(f64),
            TypeHandle::Char => parse_quote!(char),
            TypeHandle::Bool => parse_quote!(bool),
            TypeHandle::String => parse_quote!(String),
            TypeHandle::ByteArray(size) => parse_quote!([u8; #size]),
            TypeHandle::Other(name) => {
                let ident = format_ident!("{name}");
                parse_quote!(#ident)
            }
        };

        tokens.extend(std::iter::once(token));
    }
}

impl Display for TypeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeHandle::U8 => write!(f, "u8"),
            TypeHandle::U16 => write!(f, "u16"),
            TypeHandle::U32 => write!(f, "u32"),
            TypeHandle::U64 => write!(f, "u64"),
            TypeHandle::U128 => write!(f, "u128"),
            TypeHandle::I8 => write!(f, "i8"),
            TypeHandle::I16 => write!(f, "i16"),
            TypeHandle::I32 => write!(f, "i32"),
            TypeHandle::I64 => write!(f, "i64"),
            TypeHandle::I128 => write!(f, "i128"),
            TypeHandle::F32 => write!(f, "f32"),
            TypeHandle::F64 => write!(f, "f64"),
            TypeHandle::Char => write!(f, "char"),
            TypeHandle::Bool => write!(f, "bool"),
            TypeHandle::String => write!(f, "String"),
            TypeHandle::ByteArray(size) => write!(f, "[u8; {size}]"),
            TypeHandle::Other(o) => write!(f, "{o}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedType {
    Char,
    Bool,
    F32,
    F64,
    String,
    Primitive(PrimitiveType),
    Enum(Enum),
    Struct(Struct),
    Wrapper(Wrapper),
    ByteArray(usize),

    // Special generator annotations
    Remainder,
    Variable(String),

    Message(Message),
    ChunkCoder(ChunkCoder),
}

impl ResolvedType {
    pub fn name(&self) -> String {
        match self {
            ResolvedType::Char => "char".to_string(),
            ResolvedType::Bool => "bool".to_string(),
            ResolvedType::F32 => "f32".to_string(),
            ResolvedType::F64 => "f64".to_string(),
            ResolvedType::String => "String".to_string(),
            ResolvedType::Primitive(prim) => prim.name(),
            ResolvedType::Enum(e) => e.name(),
            ResolvedType::Struct(s) => s.name(),
            ResolvedType::Wrapper(w) => w.name(),
            ResolvedType::Message(m) => m.name(),
            ResolvedType::ByteArray(bytes) => format!("[u8; {bytes}]"),
            ResolvedType::Remainder => todo!(),
            ResolvedType::Variable(_) => todo!(),
            ResolvedType::ChunkCoder(_) => todo!(),
        }
    }

    pub fn get_size(&self) -> usize {
        match self {
            ResolvedType::Char | ResolvedType::Bool => 1,
            ResolvedType::F32 => 4,
            ResolvedType::F64 => 8,
            ResolvedType::String => size_of::<String>(),
            ResolvedType::Primitive(prim) => prim.get_size(),
            ResolvedType::Enum(e) => e.get_size(),
            ResolvedType::Struct(s) => s.get_size(),
            ResolvedType::Wrapper(w) => w.get_size(),
            ResolvedType::ByteArray(ba) => *ba,
            ResolvedType::Message(_) | ResolvedType::Remainder | ResolvedType::Variable(_) => {
                unreachable!()
            }
            ResolvedType::ChunkCoder(_) => todo!(),
        }
    }

    pub fn get_align(&self) -> usize {
        match self {
            ResolvedType::Char | ResolvedType::Bool | ResolvedType::ByteArray(_) => 1,
            ResolvedType::F32 => 4,
            ResolvedType::F64 => 8,
            ResolvedType::String => align_of::<String>(),
            ResolvedType::Primitive(prim) => prim.get_align(),
            ResolvedType::Enum(e) => e.get_align(),
            ResolvedType::Struct(s) => s.get_align(),
            ResolvedType::Wrapper(w) => w.get_align(),
            ResolvedType::Message(_) | ResolvedType::Remainder | ResolvedType::Variable(_) => {
                unreachable!()
            }
            ResolvedType::ChunkCoder(_) => todo!(),
        }
    }

    pub fn get_handle(&self) -> TypeHandle {
        match self {
            ResolvedType::Char => TypeHandle::Char,
            ResolvedType::Bool => TypeHandle::Bool,
            ResolvedType::F32 => TypeHandle::F32,
            ResolvedType::F64 => TypeHandle::F64,
            ResolvedType::String => TypeHandle::String,
            ResolvedType::Primitive(prim) => prim.get_handle(),
            ResolvedType::Enum(e) => TypeHandle::Other(e.name()),
            ResolvedType::Struct(s) => TypeHandle::Other(s.name()),
            ResolvedType::Wrapper(w) => TypeHandle::Other(w.name()),
            ResolvedType::ByteArray(size) => TypeHandle::ByteArray(*size),
            ResolvedType::ChunkCoder(_) => todo!(),
            ResolvedType::Remainder | ResolvedType::Message(_) | ResolvedType::Variable(_) => {
                unreachable!()
            }
        }
    }
}

impl ToTokens for ResolvedType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token = match self {
            ResolvedType::Char => parse_quote!(char),
            ResolvedType::Bool => parse_quote!(bool),
            ResolvedType::F32 => parse_quote!(f32),
            ResolvedType::F64 => parse_quote!(f64),
            ResolvedType::String => parse_quote!(String),
            ResolvedType::ByteArray(value) => parse_quote!([u8; #value]),
            ResolvedType::Primitive(prim) => parse_quote!(#prim),
            ResolvedType::Remainder | ResolvedType::Variable(_) => {
                unreachable!()
            }
            ResolvedType::Enum(e) => e.generate_code(),
            ResolvedType::Struct(s) => s.generate_code(),
            ResolvedType::Wrapper(w) => w.generate_code(),
            ResolvedType::Message(m) => m.generate_code(),
            ResolvedType::ChunkCoder(_) => todo!(),
        };

        tokens.extend(std::iter::once(token));
    }
}
