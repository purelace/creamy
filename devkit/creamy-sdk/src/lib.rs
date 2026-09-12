#![allow(clippy::inline_always)]
#![no_std]

include!(concat!(env!("OUT_DIR"), "/system.rs"));

use cbus_core::buffer::runtime::{DynIncBuf, DynOutBuf};

extern crate alloc;
pub mod api;
pub mod logging;
pub mod message;
mod sender;
pub mod state;
pub mod stream;
pub mod utils;
mod wasm;

pub mod spin {
    pub use spin::*;
}

pub use cbus_core::SubscriberId;
pub mod defines {
    pub use cbus_core::defines::*;
}

pub use sender::Sender;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: rlsf::SmallGlobalTlsf = rlsf::SmallGlobalTlsf::new();

static mut MAX_HEAP: u32 = 0;
static mut INCOMING: Option<DynIncBuf> = None;
static mut OUTGOING: Option<DynOutBuf> = None;

/// # Panics
///
/// Panics if a buffer is not initialized.
#[must_use]
#[allow(static_mut_refs)]
#[doc(hidden)]
pub fn get_incoming() -> DynIncBuf {
    unsafe {
        match INCOMING.clone() {
            Some(buf) => buf,
            None => panic!("Buffer is not initialized"),
        }
    }
}

/// # Panics
///
/// Panics if a buffer is not initialized.
#[must_use]
#[allow(static_mut_refs)]
#[doc(hidden)]
pub fn get_outgoing() -> DynOutBuf {
    unsafe {
        match OUTGOING.clone() {
            Some(buf) => buf,
            None => panic!("Buffer is not initialized"),
        }
    }
}
