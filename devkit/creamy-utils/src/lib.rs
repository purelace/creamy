#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![no_std]

extern crate alloc;

mod bstring;
mod list;
pub mod strpool;

pub use bstring::BString;

pub mod collections {
    pub use super::list::List;
}
