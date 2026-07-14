use core::{alloc::Layout, ptr::NonNull};

use cbus_core::defines::{MESSAGE_SIZE, METADATA, TARGET_ALIGN};

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
        INCOMING = cbus_core::buffer::Incoming::new(
            buffer_ptr.cast(), // The first 4 bytes is the count of a messages
            buffer_ptr.add(METADATA).cast(),
            count,
        );
    }

    buffer_ptr.as_ptr() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn internal__export_outgoing_buffer(count: u32) -> u32 {
    let buffer_ptr = alloc_buffer(count as usize);
    unsafe {
        OUTGOING = cbus_core::buffer::Outgoing::new(
            buffer_ptr.cast(), // The first 4 bytes is the count of a messages
            buffer_ptr.add(METADATA).cast(),
            count,
        );
    }

    buffer_ptr.as_ptr() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn internal__notify() {
    unsafe { export::notify() }
}
