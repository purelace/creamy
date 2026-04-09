use std::fmt::Display;

use quote::ToTokens;
use syn::Ident;

use crate::protocol::types::TypeHandle;

#[derive(Debug, Clone)]
pub enum PatternType {
    RunTime(TypeHandle, String),
    CompileTime(TypeHandle),
}

impl ToTokens for PatternType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            PatternType::RunTime(handle, _) => {
                handle.to_tokens(tokens);
            }
            PatternType::CompileTime(handle) => handle.to_tokens(tokens),
        }
    }
}

impl Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternType::RunTime(_, _) => write!(f, "variable"),
            PatternType::CompileTime(handle) => write!(f, "{handle}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: Ident,
    pub ty: PatternType,
    pub size: usize,
    //pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct ChunkCoder {
    pub name: Ident,
    pub patterns: Vec<Pattern>,
}
