#![allow(clippy::inline_always)]
#![no_std]

use cbus_core::buffer::runtime::{DynIncBuf, DynOutBuf};

extern crate alloc;
pub mod api;
mod export;
pub mod logging;
pub mod state;
pub mod stream;
pub mod utils;
mod wasm;

pub mod spin {
    pub use spin::*;
}

pub mod bus {
    pub use cbus_core::*;
}

//#[cfg(all(target_arch = "wasm32", not(target_feature = "atomics")))]
#[global_allocator]
static ALLOCATOR: rlsf::SmallGlobalTlsf = rlsf::SmallGlobalTlsf::new();

static mut MAX_HEAP: u32 = 0;
static mut INCOMING: Option<DynIncBuf> = None;
static mut OUTGOING: Option<DynOutBuf> = None;

/// # Panics
///
/// Panics if buffer is not initialized.
#[must_use]
#[allow(static_mut_refs)]
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
/// Panics if buffer is not initialized.
#[must_use]
#[allow(static_mut_refs)]
pub fn get_outgoing() -> DynOutBuf {
    unsafe {
        match OUTGOING.clone() {
            Some(buf) => buf,
            None => panic!("Buffer is not initialized"),
        }
    }
}

#[cfg(feature = "internal-testing")]
pub fn initialize_buffers(
    messages: core::num::NonZeroUsize,
) -> Result<(), core::alloc::LayoutError> {
    unsafe {
        use cbus_core::buffer::runtime::DynSharedBuf;
        INCOMING = Some(DynIncBuf::from_buf(DynSharedBuf::new(messages)?));
        OUTGOING = Some(DynOutBuf::from_buf(DynSharedBuf::new(messages)?));
    }

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/system.rs"));
