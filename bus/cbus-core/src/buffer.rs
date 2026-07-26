pub mod runtime;

use alloc::alloc::alloc_zeroed;
use core::{alloc::Layout, fmt::Debug, marker::PhantomData, num::NonZeroUsize, ptr::NonNull};

use as_guard::AsGuard;

use self::runtime::DynSharedBuf;
use crate::{
    UntypedMessage,
    defines::{MESSAGE_SIZE, METADATA, TARGET_ALIGN},
    message::TypedMessage,
};

const DANGLING_LAYOUT: Layout = match Layout::from_size_align(MESSAGE_SIZE + METADATA, TARGET_ALIGN)
{
    Ok(layout) => layout,
    Err(_) => panic!("Failed to generate layout at compile-time"),
};

//static DANGLING_SHARED_BUF: NonNull<u8> = unsafe {
//    let ptr = alloc::alloc::alloc_zeroed(DANGLING_LAYOUT);
//    NonNull::new(ptr).unwrap()
//};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SharedBufFlags: u8 {
        const SHOULD_BE_DROPPED = 0b0000_0001;
        const DANGLING = 0b0000_0010;
    }
}

pub struct RawBuf {
    count: NonNull<u32>,
    data: NonNull<UntypedMessage>,
}

impl RawBuf {
    pub const fn write_raw_mut_ptr(&mut self) -> *mut UntypedMessage {
        unsafe {
            let count = self.count.read() as usize;
            self.data.add(count).as_ptr()
        }
    }

    pub const fn set_count(&mut self, count: u32) {
        unsafe {
            self.count.write(count);
        }
    }

    #[must_use]
    pub const fn count(&self) -> u32 {
        unsafe { self.count.read() }
    }
}

pub struct RefMutBuf<'a, const SIZE: usize> {
    count: &'a mut u32,
    data: &'a mut [UntypedMessage; SIZE],
}

impl<const M: usize> RefMutBuf<'_, M> {
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

    pub const fn slice_mut(&mut self) -> &mut [UntypedMessage; M] {
        self.data
    }

    #[must_use]
    //TODO: const
    pub fn read_slice(&self) -> &[UntypedMessage] {
        let count = *self.count as usize;
        &self.data[M - count..M]
    }

    // TODO: const
    pub fn read_slice_mut(&mut self) -> &mut [UntypedMessage] {
        let count = *self.count as usize;
        &mut self.data[M - count..M]
    }

    //TODO: non zero usize
    //TODO: const
    pub fn write_slice_mut(&mut self, slice_size: usize) -> &mut [UntypedMessage] {
        let count = *self.count as usize;
        &mut self.data[count..count + slice_size]
    }
}

pub struct RefBuf<'a, const M: usize> {
    count: &'a u32,
    _data: &'a [UntypedMessage; M],
}

impl<const M: usize> RefBuf<'_, M> {
    #[must_use]
    pub const fn count(&self) -> u32 {
        *self.count
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IncBuf<const M: usize> {
    buf: SharedBuf<M>,
}

impl<const M: usize> IncBuf<M> {
    #[must_use]
    pub const fn from_buf(buf: SharedBuf<M>) -> Self {
        Self { buf }
    }

    #[must_use]
    pub const fn as_inner_ref(&self) -> &SharedBuf<M> {
        &self.buf
    }

    pub const fn as_inner_mut(&mut self) -> &mut SharedBuf<M> {
        &mut self.buf
    }

    #[must_use]
    pub const fn count(&self) -> u32 {
        self.buf.count()
    }

    pub const fn clear(&mut self) {
        self.buf.set_count(0);
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OutBuf<const M: usize> {
    buf: SharedBuf<M>,
}

impl<const M: usize> OutBuf<M> {
    #[must_use]
    pub const fn from_buf(buf: SharedBuf<M>) -> Self {
        Self { buf }
    }

    #[must_use]
    pub const fn as_inner_ref(&self) -> &SharedBuf<M> {
        &self.buf
    }

    pub const fn as_inner_mut(&mut self) -> &mut SharedBuf<M> {
        &mut self.buf
    }

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn available_space(&self) -> u32 {
        M as u32 - self.buf.count()
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
            let last_slot = self.buf.get_slice_ptr().add(M);

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
}

//TODO: check for alignment
#[derive(PartialEq, Eq)]
pub struct SharedBuf<const M: usize> {
    /// `[Count: 4-bytes]`
    /// `[Refs: 4-bytes]`
    /// `[Flags: 1-byte]`
    /// `[Padding: 55-bytes]`
    /// `[Data: M * MESSAGE_SIZE(32-bytes)]`
    ptr: NonNull<u8>,
    // Ties the struct to the thread by holding a non-thread-safe marker
    _marker: PhantomData<*const ()>,
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
    pub fn new() -> Self {
        let mut instance = unsafe {
            let raw_ptr = alloc_zeroed(Self::LAYOUT);

            let ptr = NonNull::new(raw_ptr)
                .unwrap_or_else(|| alloc::alloc::handle_alloc_error(Self::LAYOUT));
            Self {
                ptr,
                _marker: PhantomData,
            }
        };

        instance.add_reference();
        *instance.get_flags_mut() |= SharedBufFlags::SHOULD_BE_DROPPED;
        instance
    }

    #[must_use]
    pub unsafe fn from_ptr(ptr: NonNull<u8>, should_be_dropped: bool) -> Self {
        let mut instance = Self {
            ptr,
            _marker: PhantomData,
        };

        instance.reset_metadata();
        instance.add_reference();
        if should_be_dropped {
            *instance.get_flags_mut() |= SharedBufFlags::SHOULD_BE_DROPPED;
        }
        instance
    }

    #[must_use]
    pub const unsafe fn from_ptr_only(ptr: NonNull<u8>) -> Self {
        let mut instance = Self {
            ptr,
            _marker: PhantomData,
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
        let data: &[UntypedMessage; M] = unsafe {
            let array_ptr = self.get_slice_ptr().as_ptr() as *const [UntypedMessage; M];
            &*array_ptr
        };

        unsafe {
            RefBuf {
                count: self.get_count_ptr().as_ref(),
                _data: data,
            }
        }
    }

    pub const unsafe fn as_raw_buf(&mut self) -> RawBuf {
        RawBuf {
            count: self.get_count_ptr(),
            data: self.get_slice_ptr(),
        }
    }

    pub const unsafe fn as_dyn_buf(&mut self) -> DynSharedBuf {
        unsafe { DynSharedBuf::from_ptr_only(self.ptr, NonZeroUsize::new_unchecked(M)) }
    }
}

impl<const M: usize> Default for SharedBuf<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const M: usize> Clone for SharedBuf<M> {
    fn clone(&self) -> Self {
        self.add_reference();
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<const M: usize> Drop for SharedBuf<M> {
    fn drop(&mut self) {
        self.remove_reference();
        if self.is_should_be_dropped() {
            unsafe {
                alloc::alloc::dealloc(self.ptr.as_ptr(), Self::LAYOUT);
            };
        }
    }
}

impl<const M: usize> Debug for SharedBuf<M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharedBuf")
            .field("count", &self.count())
            .field("references", &self.references())
            .finish()
    }
}

#[cfg(test)]
mod tests {}
