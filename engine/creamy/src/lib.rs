#![no_std]

extern crate alloc;

include!(concat!(env!("OUT_DIR"), "/system.rs"));

mod driver;
pub mod engine;
pub mod error;
mod registry;
mod system;
mod utils;

pub mod core {
    pub use creamy_engine_core::*;
}

pub mod sdk {
    pub use creamy_sdk::*;
}
