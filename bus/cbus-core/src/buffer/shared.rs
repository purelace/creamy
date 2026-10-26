use alloc::alloc::alloc_zeroed;
use core::{alloc::Layout, num::NonZeroU32, ptr::NonNull};

use super::{
    dynamic::DynSharedBuf,
    reference::{RawBuf, RefBuf, RefMutBuf},
};
use crate::{
    UntypedMessage,
    defines::{MESSAGE_SIZE, METADATA, TARGET_ALIGN},
};

#[repr(C, align(32))]
struct AlignedBuffer([u8; 64 + 32]);

static mut TRASHCAN: AlignedBuffer = AlignedBuffer([0; 64 + 32]);

pub(super) const fn get_trashcan_ptr() -> NonNull<u8> {
    let ptr = (&raw mut TRASHCAN).cast::<u8>();
    // SAFETY: TRASHCAN is not null
    unsafe { NonNull::new_unchecked(ptr) }
}

//TODO: check for alignment
#[derive(PartialEq, Eq)]
pub(super) struct SharedBuf<const M: usize> {
    /// `[Count: 4-bytes]`
    /// `[Refs: 4-bytes]`
    /// `[Padding: 56-bytes]`
    /// `[Data: M * MESSAGE_SIZE(32-bytes)]`
    ptr: NonNull<u8>,
    should_be_dropped: bool,
}

impl<const M: usize> SharedBuf<M> {
    pub(super) const fn null() -> Self {
        Self {
            ptr: get_trashcan_ptr(),
            should_be_dropped: false,
        }
    }

    #[must_use]
    pub fn new() -> Self {
        let mut instance = unsafe {
            let raw_ptr = alloc_zeroed(Self::LAYOUT);

            let ptr = NonNull::new(raw_ptr)
                .unwrap_or_else(|| alloc::alloc::handle_alloc_error(Self::LAYOUT));
            Self {
                ptr,
                should_be_dropped: true,
            }
        };

        instance.add_reference();
        instance
    }

    pub const fn count(&self) -> u32 {
        unsafe { self.get_count_ptr().read() }
    }

    pub const fn clear(&mut self) {
        // SAFETY: 0 <= M
        unsafe {
            self.set_count(0);
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr == get_trashcan_ptr()
    }
}

impl<const M: usize> SharedBuf<M> {
    const _ASSERT_SIZE: () = const { assert!(M != 0, "generic parameter cannot be zero") };

    const SIZE: usize = M * MESSAGE_SIZE + METADATA;
    const LAYOUT: Layout = {
        match Layout::from_size_align(Self::SIZE, TARGET_ALIGN) {
            Ok(layout) => layout,
            Err(_) => panic!("Failed to generate layout at compile-time"),
        }
    };

    #[must_use]
    pub unsafe fn from_ptr(ptr: NonNull<u8>, should_be_dropped: bool) -> Self {
        let mut instance = Self {
            ptr,
            should_be_dropped,
        };

        //TODO: instance.reset_metadata();
        instance.add_reference();
        instance
    }

    #[must_use]
    pub const unsafe fn from_ptr_only(ptr: NonNull<u8>) -> Self {
        let mut instance = Self {
            ptr,
            should_be_dropped: false,
        };
        instance.add_reference();
        instance
    }

    pub(super) const fn get_count_ptr(&self) -> NonNull<u32> {
        self.ptr.cast::<u32>()
    }

    const fn get_reference_ptr(&self) -> NonNull<u32> {
        unsafe { self.get_count_ptr().add(1) }
    }

    pub const fn get_slice_ptr(&self) -> NonNull<UntypedMessage> {
        unsafe { self.ptr.add(METADATA).cast() }
    }

    pub const unsafe fn set_count(&self, value: u32) {
        unsafe {
            self.get_count_ptr().write(value);
        }
    }

    #[must_use]
    pub const fn references(&self) -> u32 {
        unsafe { self.get_reference_ptr().read() }
    }

    const fn add_reference(&self) {
        unsafe {
            let ptr = self.get_reference_ptr();
            let count = ptr.read();
            ptr.write(count + 1);
        }
    }

    const fn remove_reference(&self) {
        unsafe {
            let ptr = self.get_reference_ptr();
            let count = ptr.read();
            ptr.write(count - 1);
        }
    }

    const fn reset_metadata(&mut self) {
        unsafe {
            self.get_count_ptr().write(0);
            self.get_reference_ptr().write(0);
        };
    }

    #[must_use]
    pub const fn is_should_be_dropped(&self) -> bool {
        unsafe { self.should_be_dropped && self.get_reference_ptr().read() == 0 }
    }

    pub const fn as_mut_buf(&mut self) -> RefMutBuf<'_, M> {
        let data: &mut [UntypedMessage; M] = unsafe {
            let array_ptr = self.get_slice_ptr().as_ptr().cast::<[UntypedMessage; M]>();
            &mut *array_ptr
        };

        unsafe {
            RefMutBuf {
                count: self.get_count_ptr().as_mut(),
                data,
            }
        }
    }

    #[must_use]
    pub const fn as_ref_buf(&self) -> RefBuf<'_, M> {
        unsafe {
            RefBuf {
                count: self.get_count_ptr().as_ref(),
                _data: self.messages(),
            }
        }
    }

    pub const unsafe fn as_raw_buf(&mut self) -> RawBuf {
        RawBuf {
            count: self.get_count_ptr(),
            data: self.get_slice_ptr(),
        }
    }

    pub(super) const unsafe fn as_dyn_buf(&self) -> DynSharedBuf {
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            DynSharedBuf::from_ptr_only(self.ptr, NonZeroU32::new_unchecked(M as u32))
        }
    }

    #[must_use]
    pub const fn messages(&self) -> &[UntypedMessage; M] {
        unsafe {
            let array_ptr = self.get_slice_ptr().as_ptr() as *const [UntypedMessage; M];
            &*array_ptr
        }
    }

    pub const fn messages_mut(&mut self) -> &mut [UntypedMessage; M] {
        unsafe {
            let array_ptr = self.get_slice_ptr().as_ptr().cast::<[UntypedMessage; M]>();
            &mut *array_ptr
        }
    }
}

impl<const M: usize> Default for SharedBuf<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const M: usize> Clone for SharedBuf<M> {
    fn clone(&self) -> Self {
        if self.ptr != get_trashcan_ptr() {
            self.add_reference();
        }
        Self {
            ptr: self.ptr,
            should_be_dropped: self.should_be_dropped,
        }
    }
}

impl<const M: usize> Drop for SharedBuf<M> {
    fn drop(&mut self) {
        if self.ptr == get_trashcan_ptr() {
            return;
        }

        self.remove_reference();
        if self.is_should_be_dropped() {
            unsafe {
                alloc::alloc::dealloc(self.ptr.as_ptr(), Self::LAYOUT);
            };
        }
    }
}

impl<const M: usize> core::fmt::Debug for SharedBuf<M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharedBuf")
            .field("count", &self.count())
            .field("references", &self.references())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::ptr::NonNull;

    use crate::{
        UntypedMessage,
        buffer::shared::SharedBuf,
        defines::{MESSAGE_SIZE, METADATA},
    };

    #[test]
    fn shared_ref() {
        let buf = SharedBuf::<1024>::new();
        assert_eq!(buf.references(), 1);
        let buf0 = buf.clone();
        assert_eq!(buf.references(), 2);
        assert_eq!(buf0.references(), 2);

        drop(buf);

        assert_eq!(buf0.references(), 1);
    }

    #[test]
    #[should_panic = "zaebal"]
    fn shared_ref_in_vec() {
        let mut i = 0;
        let vec = core::iter::repeat_with(|| {
            assert!(i != 12, "zaebal");
            i += 1;
            SharedBuf::<1024>::new()
        })
        .take(24)
        .collect::<Vec<_>>();

        core::hint::black_box(&vec);

        let vec0 = vec.clone();
        drop(vec);
        core::hint::black_box(&vec0);
    }

    const fn cast(array: [u8; MESSAGE_SIZE]) -> UntypedMessage {
        unsafe { core::mem::transmute(array) }
    }

    #[test]
    fn shared_ref_from_slice() {
        const A: UntypedMessage = cast([0; MESSAGE_SIZE]);
        const B: UntypedMessage = cast([1; MESSAGE_SIZE]);
        const C: UntypedMessage = cast([2; MESSAGE_SIZE]);
        const D: UntypedMessage = cast([3; MESSAGE_SIZE]);

        #[repr(align(64))]
        struct FixedBuffer([u8; SIZE]);

        const M: usize = 4;
        const SIZE: usize = MESSAGE_SIZE * M + METADATA;
        let mut fixed_buf = FixedBuffer([0; SIZE]);
        let mut buf = unsafe {
            let ptr = NonNull::new_unchecked(fixed_buf.0.as_mut_ptr());
            SharedBuf::<M>::from_ptr(ptr, false)
        };

        let mut buf0 = buf.clone();

        let mut mut_buf = buf.as_mut_buf();
        let data = mut_buf.slice_mut();
        data[0] = A;
        data[1] = B;

        let mut mut_buf = buf0.as_mut_buf();
        let data = mut_buf.slice_mut();
        data[2] = C;
        data[3] = D;

        let mut buf = buf0.clone();

        assert_eq!(buf.as_mut_buf().slice_mut(), &mut [A, B, C, D]);
    }

    #[test]
    fn null() {
        let null = SharedBuf::<1024>::null();
        let null0 = null.clone();
        drop(null);
        let mut null = null0;

        assert_eq!(null.count(), 0);
        null.clear();
    }
}
