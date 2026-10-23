use alloc::alloc::alloc_zeroed;
use core::{alloc::Layout, num::NonZeroU32, ptr::NonNull};

use as_guard::AsGuard;

use super::shared::SharedBufFlags;
use crate::{
    UntypedMessage,
    defines::{MESSAGE_SIZE, METADATA, TARGET_ALIGN},
    message::{TypedMessage, cast_message},
};

pub(super) struct RefMutDynBuf<'a> {
    pub(super) size: NonZeroU32,
    pub(super) count: &'a mut u32,
    pub(super) data: &'a mut [UntypedMessage],
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl RefMutDynBuf<'_> {
    #[must_use]
    pub const fn count(&self) -> u32 {
        *self.count
    }

    pub const fn clear(&mut self) {
        *self.count = 0;
    }
}

#[derive(PartialEq, Eq)]
pub(super) struct DynSharedBuf {
    size: NonZeroU32,
    /// `[Count: 4-bytes]`
    /// `[Refs: 4-bytes]`
    /// `[Flags: 1-byte]`
    /// `[Padding: 55-bytes]`
    /// `[Data: M * MESSAGE_SIZE(32-bytes)]`
    ptr: NonNull<u8>,
}

impl DynSharedBuf {
    fn new(messages: NonZeroU32) -> Result<Self, core::alloc::LayoutError> {
        let instance = unsafe {
            let layout = Layout::from_size_align(
                messages.get() as usize * MESSAGE_SIZE + METADATA,
                TARGET_ALIGN,
            )?;
            let raw_ptr = alloc_zeroed(layout);

            let ptr =
                NonNull::new(raw_ptr).unwrap_or_else(|| alloc::alloc::handle_alloc_error(layout));
            Self {
                size: messages,
                ptr,
            }
        };

        instance.add_reference();
        Ok(instance)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) const fn count(&self) -> u32 {
        unsafe { self.get_count_ptr().read() }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) const fn size(&self) -> NonZeroU32 {
        self.size
    }
}

impl DynSharedBuf {
    #[must_use]
    pub(super) unsafe fn from_ptr(
        ptr: NonNull<u8>,
        messages: NonZeroU32,
        should_be_dropped: bool,
    ) -> Self {
        let mut instance = Self {
            ptr,
            size: messages,
        };

        instance.reset_metadata();
        instance.add_reference();
        if should_be_dropped {
            *instance.get_flags_mut() |= SharedBufFlags::SHOULD_BE_DROPPED;
        }
        instance
    }

    #[must_use]
    pub const unsafe fn from_ptr_only(ptr: NonNull<u8>, messages: NonZeroU32) -> Self {
        let mut instance = Self {
            ptr,
            size: messages,
        };
        instance.add_reference();
        instance
    }

    const fn get_count_ptr(&self) -> NonNull<u32> {
        self.ptr.cast::<u32>()
    }

    const fn get_reference_ptr(&self) -> NonNull<u32> {
        unsafe { self.get_count_ptr().add(1) }
    }

    const fn get_flags_ptr(&self) -> NonNull<SharedBufFlags> {
        unsafe { self.get_reference_ptr().add(1).cast() }
    }

    pub(super) const fn get_slice_ptr(&self) -> NonNull<UntypedMessage> {
        unsafe { self.ptr.add(METADATA).cast() }
    }

    const fn get_flags_mut(&mut self) -> &mut SharedBufFlags {
        unsafe { self.get_flags_ptr().as_mut() }
    }

    const fn get_flags(&self) -> SharedBufFlags {
        unsafe { self.get_flags_ptr().read() }
    }

    pub(super) const fn set_count(&self, value: u32) {
        unsafe {
            self.get_count_ptr().write(value);
        }
    }

    const fn references(&self) -> u32 {
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
            self.get_flags_ptr().write(SharedBufFlags::empty());
        };
    }

    #[must_use]
    pub const fn is_should_be_dropped(&self) -> bool {
        unsafe {
            self.get_flags().contains(SharedBufFlags::SHOULD_BE_DROPPED)
                && self.get_reference_ptr().read() == 0
        }
    }

    pub(super) const fn as_mut_buf(&mut self) -> RefMutDynBuf<'_> {
        unsafe {
            let data = core::slice::from_raw_parts_mut(
                self.get_slice_ptr().as_ptr(),
                self.size.get() as usize,
            );
            RefMutDynBuf {
                size: self.size,
                count: self.get_count_ptr().as_mut(),
                data,
            }
        }
    }

    #[must_use]
    pub const fn messages(&self) -> &[UntypedMessage] {
        unsafe {
            core::slice::from_raw_parts(self.get_slice_ptr().as_ptr(), self.size.get() as usize)
        }
    }
}

impl Clone for DynSharedBuf {
    fn clone(&self) -> Self {
        self.add_reference();
        Self {
            size: self.size,
            ptr: self.ptr,
        }
    }
}

impl Drop for DynSharedBuf {
    fn drop(&mut self) {
        self.remove_reference();
        if self.is_should_be_dropped() {
            unsafe {
                let layout = Layout::from_size_align_unchecked(
                    self.size.get() as usize * MESSAGE_SIZE + METADATA,
                    TARGET_ALIGN,
                );
                alloc::alloc::dealloc(self.ptr.as_ptr(), layout);
            };
        }
    }
}

impl core::fmt::Debug for DynSharedBuf {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharedBuf")
            .field("count", &self.count())
            .field("references", &self.references())
            .finish()
    }
}
