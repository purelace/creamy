#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//#![deny(clippy::unwrap_used)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::inline_always)]

mod bus;
pub mod config;
mod cpu;
mod driver;
mod error;
mod lookup;
mod sys;

pub mod core {
    pub use cbus_core::*;
}

pub mod defines {
    pub use cbus_core::defines::*;

    pub const MAX_SIZE: usize = isize::MAX as usize - TARGET_ALIGN;
    pub const KB: usize = 1024;
    pub const MB: usize = KB * KB;
    pub const DEFAULT_SLICES: usize = 256;
    pub const DEFAULT_SLICE_SIZE: usize = KB * 512;
    pub const METADATA: usize = 64;
    pub const SPECIAL_DATA_OFFSET: usize = 64;
    pub const TARGET_ALIGN: usize = 64;
}

pub use bus::MessageBus;
pub use driver::{BusDriver, DataIterator, DriverAnswer, OldDataIterator};
pub use error::BusError;
pub use lookup::{SubscriberLookupData, SubscriberOldLookupData};
