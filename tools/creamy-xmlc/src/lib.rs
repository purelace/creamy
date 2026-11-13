#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
//#![deny(clippy::unwrap_used)]
#![deny(clippy::todo)]
//#![deny(clippy::panic)]
//#![deny(clippy::as_conversions)]

mod compiler;
pub use compiler::compile;
pub mod tokenizer;
mod tree;

mod diagnostics;
pub mod error;
mod r#macro;
mod version;

pub use diagnostics::Diagnostics;

pub mod model {
    pub use creamy_protocol_model::*;
}

/*
 * TODO: bool type
 * TODO: C Header generation
 *
 * Pipeline:
 *     Tokenizer -> AST
 *     Resolve:
 *         Errors:
 *             Name duplicates
 *             Empty structs, fields, enums, messages, flags, arrays,
 *             Invalid size
 *             Field, variant, option count
 *
 *         Cache size and align
 *
 *
 *
 * Ограничения на протокол:
 *     i8/i16/u8/u16 - enum types
 *     28 полей на структуру
 *     28 свободных байт
 *     255 групп
 *     255 сообщений
 */

//TODO: validate size
//TODO: remove unused
//TODO: errors
//TODO: warnings
//TODO: executable
//TODO: suggest best layout
//TODO: name duplicate
//TODO: self reference
//TODO: missing type reference
