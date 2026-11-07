#![no_std]

extern crate alloc;

mod ffi;
pub mod logging;
pub mod sdk {
    pub use creamy_sdk::*;
}

use cbus_core::buffer::{DynIncBuf, DynOutBuf};

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

#[macro_export]
macro_rules! declare_plugin {
    (
        $plugin:ty
    ) => {
        #[doc(hidden)]
        static mut STATE: Option<$crate::sdk::state::PluginState<$plugin>> = None;

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn init() -> u32 {
            unsafe {
                let result = $crate::sdk::state::PluginState::new(
                    $crate::get_incoming(),
                    $crate::get_outgoing(),
                    (),
                );
                if let Some(state) = result {
                    STATE = Some(state);
                    0
                } else {
                    1
                }
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn notify() {
            unsafe {
                if let Some(state) = STATE.as_mut() {
                    state.notify();
                }
            }
        }
    };
}
