use std::{alloc::Layout, ptr::NonNull};

use crate::{core::UntypedMessage, sys::layout::alloc_pool};

pub struct MessagePool {
    layout: Layout,
    ptr: NonNull<UntypedMessage>,
    count: usize,
}

impl MessagePool {
    pub fn new(total_size: usize) -> Self {
        let (layout, ptr) = alloc_pool(total_size);
        let ptr = ptr.cast::<UntypedMessage>();

        Self {
            ptr,
            layout,
            count: 0,
        }
    }

    #[inline(always)]
    pub const fn as_slice(&self) -> &[UntypedMessage] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.count) }
    }

    #[inline(always)]
    pub const fn as_mut_slice(&mut self) -> &mut [UntypedMessage] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.count) }
    }

    //#[inline(always)]
    //pub const fn reserve(&mut self, count: usize) -> *mut UntypedMessage {
    //    unsafe {
    //        let ptr = self.ptr.add(self.count).as_ptr();
    //        self.count += count;
    //        ptr
    //    }
    //}

    #[inline(always)]
    pub const fn reserve_slice(&mut self, count: usize) -> &mut [UntypedMessage] {
        unsafe {
            let ptr = self.ptr.add(self.count).as_ptr();
            let slice = std::slice::from_raw_parts_mut(ptr, count);
            self.count += count;
            slice
        }
    }

    #[inline(always)]
    pub const fn clear(&mut self) {
        self.count = 0;
    }

    //#[inline(always)]
    //pub const fn count(&self) -> usize {
    //    self.count
    //}

    //#[inline(always)]
    //pub const fn ptr_at(&mut self, count: usize) -> *mut UntypedMessage {
    //    unsafe { self.ptr.add(count).as_ptr() }
    //}

    pub const fn slice(&mut self, len: usize, ptr_location: usize) -> &[UntypedMessage] {
        unsafe {
            let ptr = self.ptr.add(ptr_location).as_ptr();
            std::slice::from_raw_parts(ptr, len)
        }
    }
}

impl Drop for MessagePool {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.ptr.as_ptr().cast::<u8>(), self.layout);
        }
    }
}
