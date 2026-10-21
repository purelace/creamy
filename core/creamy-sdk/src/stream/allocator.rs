/*
use core::{alloc::Allocator, num::NonZeroUsize, ptr::NonNull};

enum MemoryRegion {
    Static { start: usize, end: NonZeroUsize },
    Heap { ptr: NonNull<[u8]> },
}

fn test() {
    let a: MemoryRegion;
    let b: Option<MemoryRegion>;
}

pub struct CacheAllocator {
    regions: [Option<MemoryRegion>; 128],
    memory: [u8; 1024],
}

impl CacheAllocator {
    pub const fn new() -> Self {
        Self {
            regions: [None; 128],
            memory: [0; 1024],
        }
    }
}

unsafe impl Allocator for CacheAllocator {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        layout.size()
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {}
}
*/
