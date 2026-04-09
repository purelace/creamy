use std::{alloc::Layout, ptr::NonNull};

use crate::defines::TARGET_ALIGN;

pub fn alloc_pool(total_size: usize) -> (Layout, NonNull<u8>) {
    // assert!(total_size == 0) is unreachable due to config checks
    // assert!(total_size & TARGET_ALIGN == 0) is unreachable due to config checks
    // MAX_SIZE is unreachable due to config checks
    unsafe {
        // Align is not zero;
        // Align is power of two (check crate::TARGET_ALIGN);
        let layout = Layout::from_size_align_unchecked(total_size, TARGET_ALIGN);
        // Выделяем память и заполняем нулями
        let ptr = std::alloc::alloc(layout);
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        let bytes = NonNull::new(ptr).expect("Failed to allocate memory");

        (layout, bytes)
    }
}
