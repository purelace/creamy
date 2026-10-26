use core::{mem::ManuallyDrop, num::NonZeroU32, ptr::NonNull};

use super::{
    dynamic::DynSharedBuf,
    reference::{RawBuf, RefBuf, RefMutBuf},
    shared::SharedBuf,
};
use crate::UntypedMessage;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncBuf<const M: usize> {
    buf: SharedBuf<M>,
}

impl<const M: usize> Default for IncBuf<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const M: usize> IncBuf<M> {
    #[must_use]
    pub const fn null() -> Self {
        Self {
            buf: SharedBuf::null(),
        }
    }

    #[must_use]
    pub fn new() -> Self {
        Self {
            buf: SharedBuf::new(),
        }
    }

    #[must_use]
    pub const fn count(&self) -> u32 {
        self.buf.count()
    }

    pub const fn clear(&mut self) {
        self.buf.clear();
    }

    pub const fn pop(&mut self) -> Option<UntypedMessage> {
        if self.count() == 0 {
            return None;
        }

        let buffer = self.buf.as_mut_buf();
        let start = buffer.count() - 1;
        *buffer.count -= 1;

        Some(buffer.data[start as usize])
    }

    pub fn pop_all(&mut self) -> core::iter::Rev<core::slice::Iter<'_, UntypedMessage>> {
        let count = self.buf.count() as usize;
        let mut buffer = self.buf.as_mut_buf();
        buffer.clear();
        buffer.data[..count].iter().rev()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<const M: usize> IncBuf<M> {
    #[must_use]
    #[doc(hidden)]
    pub unsafe fn from_ptr(ptr: NonNull<u8>, should_be_dropped: bool) -> Self {
        unsafe {
            Self {
                buf: SharedBuf::from_ptr(ptr, should_be_dropped),
            }
        }
    }

    #[must_use]
    #[doc(hidden)]
    pub const fn as_ref_buf(&self) -> RefBuf<'_, M> {
        self.buf.as_ref_buf()
    }

    #[doc(hidden)]
    pub const unsafe fn as_mut_buf(&mut self) -> RefMutBuf<'_, M> {
        self.buf.as_mut_buf()
    }

    #[doc(hidden)]
    pub const unsafe fn as_raw_buf(&mut self) -> RawBuf {
        unsafe { self.buf.as_raw_buf() }
    }

    #[must_use]
    #[doc(hidden)]
    pub const fn as_dyn_buf(&self) -> DynIncBuf {
        unsafe { DynIncBuf::from_shared_buf(self.buf.as_dyn_buf()) }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynIncBuf {
    buf: DynSharedBuf,
}

impl DynIncBuf {
    #[must_use]
    pub unsafe fn from_ptr(
        ptr: NonNull<u8>,
        messages: NonZeroU32,
        should_be_dropped: bool,
    ) -> Self {
        unsafe {
            Self {
                buf: DynSharedBuf::from_ptr(ptr, messages, should_be_dropped),
            }
        }
    }

    #[must_use]
    const unsafe fn from_ptr_only(ptr: NonNull<u8>, messages: NonZeroU32) -> Self {
        unsafe {
            Self {
                buf: DynSharedBuf::from_ptr_only(ptr, messages),
            }
        }
    }

    #[must_use]
    pub(super) const unsafe fn from_shared_buf(buf: DynSharedBuf) -> Self {
        Self { buf }
    }

    pub const fn clear(&mut self) {
        self.buf.set_count(0);
    }

    #[must_use]
    pub const fn count(&self) -> u32 {
        self.buf.count()
    }

    pub const fn pop(&mut self) -> Option<UntypedMessage> {
        if self.count() == 0 {
            return None;
        }

        let buffer = self.buf.as_mut_buf();
        let start = buffer.count() - 1;
        *buffer.count -= 1;

        Some(buffer.data[start as usize])
    }

    pub fn pop_all(&mut self) -> core::iter::Rev<core::slice::Iter<'_, UntypedMessage>> {
        let count = self.buf.count() as usize;
        let mut buffer = self.buf.as_mut_buf();
        buffer.clear();
        buffer.data[..count].iter().rev()
    }
}
