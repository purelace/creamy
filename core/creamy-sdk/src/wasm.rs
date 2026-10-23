use core::{alloc::Layout, num::NonZeroU32, ptr::NonNull};

use cbus_core::defines::{MESSAGE_SIZE, METADATA, TARGET_ALIGN};

use crate::{INCOMING, MAX_HEAP, OUTGOING};

#[unsafe(no_mangle)]
pub extern "C" fn internal__init_plugin(max_heap: u32) -> u32 {
    unsafe {
        MAX_HEAP = max_heap;
        init()
    }
}

fn alloc_buffer(count: usize) -> NonNull<u8> {
    let buffer_size = count * MESSAGE_SIZE + METADATA;
    let layout = Layout::from_size_align(buffer_size, TARGET_ALIGN).unwrap();
    unsafe {
        let ptr = alloc::alloc::alloc(layout);
        if let Some(ptr) = NonNull::new(ptr) {
            ptr
        } else {
            alloc::alloc::handle_alloc_error(layout);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn internal__export_incoming_buffer(count: u32) -> u32 {
    let buffer_ptr = alloc_buffer(count as usize);
    unsafe {
        INCOMING = Some(cbus_core::buffer::DynIncBuf::from_ptr(
            buffer_ptr,
            NonZeroU32::new_unchecked(count),
            false,
        ));
    }

    buffer_ptr.as_ptr() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn internal__export_outgoing_buffer(count: u32) -> u32 {
    let buffer_ptr = alloc_buffer(count as usize);
    unsafe {
        OUTGOING = Some(cbus_core::buffer::DynOutBuf::from_ptr(
            buffer_ptr,
            NonZeroU32::new_unchecked(count),
            false,
        ));
    }

    buffer_ptr.as_ptr() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn internal__notify() {
    unsafe { notify() }
}

unsafe extern "C" {
    pub fn init() -> u32;
    pub fn notify();
}
