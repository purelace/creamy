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

#[macro_export]
macro_rules! define_engine {
    (
        max_messages: $mm: expr,
        max_groups: $mg: expr,
        max_subscribers: $ms: expr,
    ) => {
        extern crate alloc;
        pub const MAX_MESSAGES: usize = $mm;
        pub const MAX_GROUPS: usize = $mg;
        pub const MAX_SUBSCRIBERS: usize = $ms;
        pub type IncBuf = $crate::core::bus::core::buffer::IncBuf<$mm>;
        pub type OutBuf = $crate::core::bus::core::buffer::OutBuf<$mm>;
        pub type PluginEngine<R, L, H> = $crate::engine::PluginEngine<R, L, H, $mm, $mg, $ms>;
    };
}
