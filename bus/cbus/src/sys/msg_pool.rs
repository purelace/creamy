use core::{alloc::Layout, ptr::NonNull};

use cbus_core::defines::{MESSAGE_SIZE, TARGET_ALIGN};

use crate::{core::UntypedMessage, sys::layout::alloc_pool};

#[derive(Debug)]
pub struct MessagePool<const MM: usize, const MS: usize> {
    ptr: NonNull<UntypedMessage>,
    count: usize,
}

impl<const MM: usize, const MS: usize> MessagePool<MM, MS> {
    const LAYOUT: Layout = match Layout::from_size_align(MS * MM * MESSAGE_SIZE, TARGET_ALIGN) {
        Ok(v) => v,
        Err(_) => panic!("Failed to generate layout at compile-time"),
    };

    pub fn new() -> Self {
        let ptr = alloc_pool(Self::LAYOUT);
        let ptr = ptr.cast::<UntypedMessage>();

        Self { ptr, count: 0 }
    }

    #[inline(always)]
    pub const fn as_slice(&self) -> &[UntypedMessage] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.count) }
    }

    #[inline(always)]
    pub const fn as_mut_slice(&mut self) -> &mut [UntypedMessage] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.count) }
    }

    #[inline(always)]
    pub fn reserve_slice(&mut self, count: usize) -> &mut [UntypedMessage] {
        unsafe {
            let ptr = self.ptr.add(self.count).as_ptr();
            let slice = core::slice::from_raw_parts_mut(ptr, count);
            self.count += count;
            slice
        }
    }

    #[inline(always)]
    pub const fn clear(&mut self) {
        self.count = 0;
    }

    pub const fn slice(&mut self, len: u16, offset: u16) -> &[UntypedMessage] {
        unsafe {
            let ptr = self.ptr.add(offset as usize).as_ptr();
            core::slice::from_raw_parts(ptr, len as usize)
        }
    }
}

impl<const MM: usize, const MS: usize> Drop for MessagePool<MM, MS> {
    fn drop(&mut self) {
        unsafe {
            alloc::alloc::dealloc(self.ptr.as_ptr().cast::<u8>(), Self::LAYOUT);
        }
    }
}
