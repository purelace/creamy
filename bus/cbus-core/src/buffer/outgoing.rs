use core::{num::NonZeroU32, ptr::NonNull};

use as_guard::AsGuard;

use super::{
    dynamic::DynSharedBuf,
    reference::{RawBuf, RefBuf, RefMutBuf},
    shared::SharedBuf,
};
use crate::{
    UntypedMessage,
    message::{TypedMessage, cast_message},
};

#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OutBuf<const M: usize> {
    buf: SharedBuf<M>,
}

impl<const M: usize> OutBuf<M> {
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn available_space(&self) -> u32 {
        M as u32 - self.buf.count()
    }

    #[must_use]
    pub const fn messages(&self) -> &[UntypedMessage] {
        self.buf.messages()
    }

    #[must_use]
    pub const fn count(&self) -> u32 {
        self.buf.count()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count() == 0
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
                ptr.write(cast_message(msg));
                ptr = ptr.add(1);
            }
        }
        true
    }

    //если места недостаточно, то ни одно из сообщений отправлено не будет.

    /// # Returns
    ///
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

    /// Отправляет одно сообщение
    ///
    /// # Returns
    ///
    /// Возвращает bool которое указывает на то, хватает ли места в буфере.
    pub fn send(&mut self, message: impl TypedMessage) -> bool {
        self.send_many_iter_exact(core::iter::once(message))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<const M: usize> OutBuf<M> {
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

    const fn reserve(&mut self, count: u32) {
        // SAFETY: not safety
        unsafe {
            self.buf.set_count(self.buf.count() + count);
        }
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
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynOutBuf {
    buf: DynSharedBuf,
}

impl DynOutBuf {
    //pub unsafe fn null() -> Result<Self, core::alloc::LayoutError> {
    //    const MESSAGES: NonZeroU32 = NonZeroU32::new(1).unwrap();
    //    let buf = DynSharedBuf::new(MESSAGES)?;
    //    Ok(Self { buf })
    //}

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

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn available_space(&self) -> u32 {
        self.buf.size().get() - self.buf.count()
    }

    #[must_use]
    pub const fn count(&self) -> u32 {
        self.buf.count()
    }

    const fn reserve(&mut self, count: u32) {
        self.buf.set_count(self.buf.count() + count);
    }

    #[must_use]
    pub const fn messages(&self) -> &[UntypedMessage] {
        self.buf.messages()
    }

    #[inline]
    const fn write_ptr(&self) -> NonNull<UntypedMessage> {
        unsafe {
            // Перемещаем указатель на последний слот.
            // Capacity указывает на PADDING, поэтому вычитаем 1.
            // см. PADDING
            let last_slot = self.buf.get_slice_ptr().add(self.buf.size().get() as usize);

            // Вычитаем уже занятые слоты, чтобы не затереть данные
            last_slot.sub(self.buf.count() as usize + 1)
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
                ptr.write(cast_message(msg));
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
        if self.count() >= self.buf.size().get() {
            return None;
        }

        let message_ptr = self.write_ptr();
        self.buf.set_count(self.buf.count() + 1);
        Some(message_ptr)
    }
}
