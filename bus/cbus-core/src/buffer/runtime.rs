use alloc::alloc::alloc_zeroed;
use core::{alloc::Layout, num::NonZeroUsize, ptr::NonNull};

use as_guard::AsGuard;

use super::{SharedBuf, SharedBufFlags};
use crate::{
    UntypedMessage,
    defines::{MESSAGE_SIZE, METADATA, TARGET_ALIGN},
    message::TypedMessage,
};

pub struct RefMutDynBuf<'a> {
    size: NonZeroUsize,
    count: &'a mut u32,
    data: &'a mut [UntypedMessage],
}

impl RefMutDynBuf<'_> {
    pub const fn set_count(&mut self, count: u32) {
        *self.count = count;
    }

    pub const fn add_count(&mut self, count: u32) {
        *self.count = *self.count + count;
    }

    #[must_use]
    pub const fn count(&self) -> u32 {
        *self.count
    }

    #[must_use]
    //TODO: const
    pub fn read_slice(&self) -> &[UntypedMessage] {
        let count = *self.count as usize;
        let size = self.size.get();
        &self.data[size - count..size]
    }

    // TODO: const
    pub fn read_slice_mut(&mut self) -> &mut [UntypedMessage] {
        let count = *self.count as usize;
        let size = self.size.get();
        &mut self.data[size - count..size]
    }

    //TODO: non zero usize
    //TODO: const
    //pub fn write_slice_mut(&mut self, slice_size: usize) -> &mut [UntypedMessage] {
    //    let count = *self.count as usize;
    //    &mut self.data[count..count + slice_size]
    //}
}

#[derive(PartialEq, Eq)]
pub struct DynSharedBuf {
    size: NonZeroUsize,
    /// `[Count: 4-bytes]`
    /// `[Refs: 4-bytes]`
    /// `[Flags: 1-byte]`
    /// `[Padding: 55-bytes]`
    /// `[Data: M * MESSAGE_SIZE(32-bytes)]`
    ptr: NonNull<u8>,
}

impl DynSharedBuf {
    pub fn new(messages: NonZeroUsize) -> Result<Self, core::alloc::LayoutError> {
        let instance = unsafe {
            let layout =
                Layout::from_size_align(messages.get() * MESSAGE_SIZE + METADATA, TARGET_ALIGN)?;
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

    #[must_use]
    pub unsafe fn from_ptr(
        ptr: NonNull<u8>,
        messages: NonZeroUsize,
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
    pub const unsafe fn from_ptr_only(ptr: NonNull<u8>, messages: NonZeroUsize) -> Self {
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

    const fn get_slice_ptr(&self) -> NonNull<UntypedMessage> {
        unsafe { self.ptr.add(METADATA).cast() }
    }

    const fn get_flags_mut(&mut self) -> &mut SharedBufFlags {
        unsafe { self.get_flags_ptr().as_mut() }
    }

    const fn get_flags(&self) -> SharedBufFlags {
        unsafe { self.get_flags_ptr().read() }
    }

    const fn count(&self) -> u32 {
        unsafe { self.get_count_ptr().read() }
    }

    const fn set_count(&self, value: u32) {
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

    pub const fn as_mut_buf(&mut self) -> RefMutDynBuf<'_> {
        unsafe {
            let data =
                core::slice::from_raw_parts_mut(self.get_slice_ptr().as_ptr(), self.size.get());
            RefMutDynBuf {
                size: self.size,
                count: self.get_count_ptr().as_mut(),
                data,
            }
        }
    }

    pub const unsafe fn as_const_sized<const M: usize>(&mut self) -> SharedBuf<M> {
        unsafe { SharedBuf::from_ptr_only(self.ptr) }
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
                    self.size.get() * MESSAGE_SIZE + METADATA,
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

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynIncBuf {
    buf: DynSharedBuf,
}

impl DynIncBuf {
    pub unsafe fn null() -> Result<Self, core::alloc::LayoutError> {
        const MESSAGES: NonZeroUsize = NonZeroUsize::new(1).unwrap();
        let buf = DynSharedBuf::new(MESSAGES)?;
        Ok(Self { buf })
    }

    #[must_use]
    pub const fn from_buf(buf: DynSharedBuf) -> Self {
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
        let start = buffer.count() as usize - 1;
        *buffer.count -= 1;

        Some(buffer.data[start])
    }

    pub fn pop_all(&mut self) -> &[UntypedMessage] {
        let count = self.buf.count() as usize;
        if count == 0 {
            return &[];
        }

        let mut buffer = self.buf.as_mut_buf();
        buffer.set_count(0);

        &buffer.data[..count]
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynOutBuf {
    buf: DynSharedBuf,
}

impl DynOutBuf {
    pub unsafe fn null() -> Result<Self, core::alloc::LayoutError> {
        const MESSAGES: NonZeroUsize = NonZeroUsize::new(1).unwrap();
        let buf = DynSharedBuf::new(MESSAGES)?;
        Ok(Self { buf })
    }

    #[must_use]
    pub const fn from_buf(buf: DynSharedBuf) -> Self {
        Self { buf }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn available_space(&self) -> u32 {
        self.buf.size.get() as u32 - self.buf.count()
    }

    #[must_use]
    pub const fn count(&self) -> u32 {
        self.buf.count()
    }

    const fn reserve(&mut self, count: u32) {
        self.buf.set_count(self.buf.count() + count);
    }

    #[inline]
    const fn write_ptr(&self) -> NonNull<UntypedMessage> {
        unsafe {
            // Перемещаем указатель на последний слот.
            // Capacity указывает на PADDING, поэтому вычитаем 1.
            // см. PADDING
            let last_slot = self.buf.get_slice_ptr().add(self.buf.size.get());

            // Вычитаем уже занятые слоты, чтобы не затереть данные
            last_slot.sub(self.buf.count() as usize)
        }
    }

    #[inline]
    pub fn send_many_iter_with_count<T, I>(&mut self, iter: I, count: usize) -> bool
    where
        T: TypedMessage,
        I: IntoIterator<Item = T>,
        <I as core::iter::IntoIterator>::IntoIter: core::iter::DoubleEndedIterator,
    {
        let iter = iter.into_iter();

        // Проверяем наличие свободного места
        if (self.available_space() as usize) < count {
            return false;
        }

        // Заранее резервируем пространство
        self.reserve(count.safe_as());

        unsafe {
            // Получаем указатель на начало свободной зоны
            // Мы не применяем смещение к указателю, так как он уже смещен в начало
            // зарезервированной памяти
            let mut ptr = self.write_ptr().as_ptr();

            for msg in iter.rev() {
                ptr.write(msg.cast());
                ptr = ptr.add(1);
            }
        }
        true
    }

    /// # Returns
    /// Возвращает bool которое указывает на то, хватает ли места в буфере.
    #[inline]
    pub fn send_many_iter_exact<T, I>(&mut self, iter: I) -> bool
    where
        T: TypedMessage,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
        <I as core::iter::IntoIterator>::IntoIter: core::iter::DoubleEndedIterator,
    {
        let iter = iter.into_iter();
        let count = iter.len();
        self.send_many_iter_with_count(iter, count)
    }

    /// Returns `false` if the buffer is full
    #[must_use]
    pub const fn send<M: TypedMessage>(&mut self, message: &M) -> bool {
        self.send_internal_typed(message)
    }

    // TODO: fix bugs
    /// Returns `false` if the buffer is full
    #[must_use]
    pub const fn send_untyped(&mut self, message: &UntypedMessage) -> bool {
        self.send_internal_untyped(message)
    }

    #[must_use]
    pub(crate) const fn send_internal_typed<M: TypedMessage>(&mut self, message: &M) -> bool {
        let untyped = unsafe { &*core::ptr::from_ref::<M>(message).cast::<UntypedMessage>() };
        self.send_internal_untyped(untyped)
    }

    #[must_use]
    pub(crate) const fn send_internal_untyped(&mut self, message: &UntypedMessage) -> bool {
        let Some(dst_ptr) = self.next_message_ptr() else {
            return false;
        };

        unsafe {
            dst_ptr.write(*message);
        }

        true
    }

    #[must_use]
    const fn next_message_ptr(&mut self) -> Option<NonNull<UntypedMessage>> {
        if self.count() as usize >= self.buf.size.get() {
            return None;
        }

        let message_ptr = self.write_ptr();
        self.buf.set_count(self.buf.count() + 1);
        Some(message_ptr)
    }
}
