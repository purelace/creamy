use core::{alloc::Layout, marker::PhantomData, ptr::NonNull};

use cbus_core::defines::{MESSAGE_SIZE, TARGET_ALIGN};

use crate::{config::BusConfig, core::UntypedMessage, sys::layout::alloc_pool};

#[derive(Debug)]
pub struct MessagePool<C: BusConfig> {
    ptr: NonNull<UntypedMessage>,
    count: usize,
    _phantom: PhantomData<C>,
}

impl<C: BusConfig> MessagePool<C> {
    const LAYOUT: Layout = match Layout::from_size_align(
        C::MAX_SUBSCRIBERS.get() as usize * C::MAX_MESSAGES.get() as usize * MESSAGE_SIZE,
        TARGET_ALIGN,
    ) {
        Ok(v) => v,
        Err(_) => panic!("Failed to generate layout at compile-time"),
    };

    pub fn new() -> Self {
        let ptr = alloc_pool(Self::LAYOUT);
        let ptr = ptr.cast::<UntypedMessage>();

        Self {
            ptr,
            count: 0,
            _phantom: PhantomData,
        }
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
            //println!("count: {}, add: {count}", self.count);
            let slice = core::slice::from_raw_parts_mut(ptr, count);
            self.count += count;
            slice
        }
    }

    #[inline(always)]
    pub const fn clear(&mut self) {
        self.count = 0;
    }

    pub const fn count(&self) -> usize {
        self.count
    }

    pub const fn slice(&mut self, len: usize, ptr_location: usize) -> &[UntypedMessage] {
        unsafe {
            let ptr = self.ptr.add(ptr_location).as_ptr();
            core::slice::from_raw_parts(ptr, len)
        }
    }
}

impl<C: BusConfig> Drop for MessagePool<C> {
    fn drop(&mut self) {
        unsafe {
            alloc::alloc::dealloc(self.ptr.as_ptr().cast::<u8>(), Self::LAYOUT);
        }
    }
}
