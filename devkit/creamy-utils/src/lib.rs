#![no_std]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

extern crate alloc;

mod bstring;
mod list;
pub mod strpool;

pub use bstring::BString;

pub mod collections {
    pub use super::list::List;
}
