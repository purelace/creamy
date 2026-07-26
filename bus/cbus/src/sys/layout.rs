use core::{alloc::Layout, ptr::NonNull};

pub fn alloc_pool(layout: Layout) -> NonNull<u8> {
    unsafe {
        let raw_ptr = alloc::alloc::alloc(layout);
        NonNull::new(raw_ptr).unwrap_or_else(|| alloc::alloc::handle_alloc_error(layout))
    }
}
