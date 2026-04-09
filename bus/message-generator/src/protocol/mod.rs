mod definitions;
mod primitive;
mod resolved;
mod table;
mod types;

use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Duplicate variant '{duplicate}' in enum '{src}'")]
    DuplicateVariant { duplicate: String, src: String },

    #[error("Duplicate field '{duplicate}' in '{src}'")]
    DuplicateField { duplicate: String, src: String },

    #[error("Missing attribute '{name}' in field '{src}'")]
    MissingAttributeInField { name: String, src: String },

    #[error("Missing attribute '{name}' in '{src}'")]
    MissingAttribute { name: String, src: String },

    #[error("Unknown '{0}' tag")]
    UnknownTag(String),

    #[error("Cannot resolve types: missing '{0}' type")]
    MissingType(String),

    #[error("Cannot resolve types: type '{0}' already defined")]
    AlreadyDefined(String),
}

use proc_macro2::TokenStream;
use quote::quote;

use crate::protocol::{definitions::Definitions, table::resolve};

pub fn generate_code_from_xml(content: &str) -> Result<TokenStream> {
    let definitions = Definitions::parse_from_xml(content)?;
    let (protocol, mut types) = resolve(definitions);
    types.filter();
    Ok(quote! {
        pub mod #protocol {
            #types
        }
    })
}
