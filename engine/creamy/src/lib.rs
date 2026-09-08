#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![no_std]

extern crate alloc;

mod driver;
pub mod engine;
mod registry;

pub mod core {
    pub use creamy_engine_core::*;
}

pub mod sdk {
    pub use creamy_sdk::*;
}
