use core::{alloc::Layout, num::NonZeroUsize, ptr::NonNull};

use cbus_core::{
    buffer::runtime::DynSharedBuf,
    defines::{MESSAGE_SIZE, METADATA, TARGET_ALIGN},
};

use crate::{INCOMING, MAX_HEAP, OUTGOING, export};

#[unsafe(no_mangle)]
pub extern "C" fn internal__init_plugin(max_heap: u32) -> u32 {
    unsafe {
        MAX_HEAP = max_heap;
        export::init()
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
        INCOMING = Some(cbus_core::buffer::runtime::DynIncBuf::from_buf(
            DynSharedBuf::from_ptr(
                buffer_ptr,
                NonZeroUsize::new_unchecked(count as usize),
                false,
            ),
        ));
    }

    buffer_ptr.as_ptr() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn internal__export_outgoing_buffer(count: u32) -> u32 {
    let buffer_ptr = alloc_buffer(count as usize);
    unsafe {
        OUTGOING = Some(cbus_core::buffer::runtime::DynOutBuf::from_buf(
            DynSharedBuf::from_ptr(
                buffer_ptr,
                NonZeroUsize::new_unchecked(count as usize),
                false,
            ),
        ));
    }

    buffer_ptr.as_ptr() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn internal__notify() {
    unsafe { export::notify() }
}
