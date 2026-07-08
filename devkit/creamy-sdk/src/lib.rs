#![no_std]

extern crate alloc;
pub mod api;
pub mod export;
mod logging;
pub mod stream;
pub mod utils;
mod wasm;

use cbus_core::buffer::{Incoming, Outgoing};

pub mod bus {
    pub use cbus_core::*;
}

//#[cfg(all(target_arch = "wasm32", not(target_feature = "atomics")))]
#[global_allocator]
static ALLOCATOR: rlsf::SmallGlobalTlsf = rlsf::SmallGlobalTlsf::new();

static mut MAX_HEAP: u32 = 0;
static mut INCOMING: Incoming = Incoming::null();
static mut OUTGOING: Outgoing = Outgoing::null();

#[must_use]
pub const fn get_incoming() -> Incoming {
    unsafe { INCOMING }
}

#[must_use]
pub const fn get_outgoing() -> Outgoing {
    unsafe { OUTGOING }
}

include!(concat!(env!("OUT_DIR"), "/system.rs"));
