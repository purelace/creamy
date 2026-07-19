#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

mod array;
mod bstring;
mod list;
pub mod strpool;

pub use bstring::BString;

pub mod collections {
    pub use super::{array::Array, list::List};
}
