#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
//#![deny(clippy::unwrap_used)]
//#![deny(clippy::todo)]
//#![deny(clippy::panic)]
//#![deny(clippy::as_conversions)]

mod ast;
mod compiler;
pub mod symbols;
pub mod table;
mod tokenizer;
pub mod utils;
mod version;

mod diagnostics;
pub mod error;

pub use compiler::{Api, SemanticModel, get_meta};
pub use diagnostics::Diagnostics;
//TODO: bool type
//TODO: validate size
//TODO: remove unused
//TODO: errors
//TODO: warnings
//TODO: executable
//TODO: suggest best layout
//TODO: name duplicate
//TODO: self reference
//TODO: missing type reference

pub const MAX_PAYLOAD: u64 = 28;
pub const HEADER_BYTES: u64 = 4;
