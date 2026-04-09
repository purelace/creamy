use core::{marker::PhantomData, ptr::NonNull, slice};

use as_guard::AsGuard;

use crate::{UntypedMessage, message::TypedMessage};

pub type Outgoing = Buffer<Write>;
pub type Incoming = Buffer<Read>;

//#[repr(C)]
//struct Header {
//    count: u32,
//    ignore: u32,
//    _padding: [u8; 56],
//}

pub struct Buffer<O> {
    count: NonNull<u32>,
    slice: NonNull<UntypedMessage>,
    capacity: u32,
    _phantom: PhantomData<O>,
}

unsafe impl<O> Send for Buffer<O> {}
unsafe impl<O> Sync for Buffer<O> {}

//impl<O> Drop for Buffer<O> {
//    fn drop(&mut self) {
//        unsafe {
//            let header = &mut *self.count.as_ptr().cast::<Header>();
//            header.count = 0;
//            header.ignore = 1;
//        }
//    }
//}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<O> Buffer<O> {
    #[must_use]
    #[inline(always)]
    pub const fn new(count: NonNull<u32>, slice: NonNull<UntypedMessage>, capacity: u32) -> Self {
        Buffer {
            count,
            slice,
            capacity,
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub const fn clear(&mut self) {
        unsafe {
            self.count.write(0);
        }
    }

    #[must_use]
    #[inline(always)]
    pub const fn count(&self) -> u32 {
        unsafe { self.count.read() }
    }

    #[must_use]
    #[inline(always)]
    pub const fn available_space(&self) -> u32 {
        unsafe { self.capacity - self.count.read() }
    }

    #[must_use]
    #[inline(always)]
    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    #[inline(always)]
    const fn increment_count(&mut self) {
        unsafe {
            self.count.write(self.count() + 1);
        }
    }

    #[inline(always)]
    const fn reserve(&mut self, count: u32) {
        unsafe {
            self.count.write(self.count() + count);
        }
    }

    #[inline(always)]
    const fn decrement_count(&mut self) {
        unsafe {
            self.count.write(self.count() - 1);
        }
    }
}

impl<O> Buffer<O> {
    #[must_use]
    #[inline(always)]
    const fn next_message_ptr(&mut self) -> Option<NonNull<UntypedMessage>> {
        if self.count() >= self.capacity {
            return None;
        }

        let message_ptr = self.write_ptr();
        self.increment_count();
        Some(message_ptr)
    }

    #[must_use]
    #[inline(always)]
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

    pub(crate) const fn read_internal(&self) -> &[UntypedMessage] {
        unsafe { core::slice::from_raw_parts(self.write_ptr().as_ptr(), self.count() as usize) }
    }

    #[must_use]
    #[inline(always)]
    pub const fn as_slice(&self) -> &[UntypedMessage] {
        self.read_internal()
    }

    #[inline(always)]
    const fn write_ptr(&self) -> NonNull<UntypedMessage> {
        unsafe {
            // Перемещаем указатель на последний слот.
            // Capacity указывает на PADDING, поэтому вычитаем 1.
            // см. PADDING
            let last_slot = self.slice.add((self.capacity as usize) - 1);

            // Вычитаем уже занятые слоты, чтобы не затереть данные
            last_slot.sub(self.count() as usize)
        }
    }
}

pub struct Write;
impl Buffer<Write> {
    /// Return false if the buffer is full
    #[must_use]
    pub const fn send<M: TypedMessage>(&mut self, message: &M) -> bool {
        self.send_internal_typed(message)
    }

    //TODO fix bugs
    /// Return false if the buffer is full
    #[must_use]
    pub const fn send_untyped(&mut self, message: &UntypedMessage) -> bool {
        self.send_internal_untyped(message)
    }

    pub const fn as_mut_slice(&mut self) -> &mut [UntypedMessage] {
        unsafe { slice::from_raw_parts_mut(self.write_ptr().as_ptr(), self.count() as usize) }
    }

    #[inline(never)]
    pub fn send_many_iter_with_count<I>(&mut self, iter: I, count: usize) -> bool
    where
        I: IntoIterator<Item = UntypedMessage>,
        <I as core::iter::IntoIterator>::IntoIter: core::iter::DoubleEndedIterator,
    {
        let iter = iter.into_iter();

        // Проверяем наличие свободного места
        if (self.available_space() as usize) < count {
            return false;
        }

        unsafe {
            // Заранее резервируем пространство
            self.reserve(count.safe_as());

            // Получаем указатель на начало свободной зоны
            // Мы не применяем смещение к указателю, так как он уже смещен в начало
            // зарезервированной памяти
            let mut ptr = self.write_ptr().as_ptr();

            for msg in iter.rev() {
                ptr.write(msg);
                ptr = ptr.add(1);
            }
        }
        true
    }

    /// # Returns
    /// Возвращает bool которое указывает на то, хватает ли места в буфере.
    #[inline(always)]
    pub fn send_many_iter_exact<I>(&mut self, iter: I) -> bool
    where
        I: IntoIterator<Item = UntypedMessage>,
        I::IntoIter: ExactSizeIterator,
        <I as core::iter::IntoIterator>::IntoIter: core::iter::DoubleEndedIterator,
    {
        let iter = iter.into_iter();
        let count = iter.len();
        self.send_many_iter_with_count(iter, count)
    }
}

pub struct Read;
impl Buffer<Read> {
    pub const fn pop(&mut self) -> Option<UntypedMessage> {
        if self.count() == 0 {
            return None;
        }

        let start = self.count() as usize - 1;
        unsafe {
            let message_ptr = self.slice.add(start);
            self.decrement_count();
            Some(*message_ptr.as_ptr())
        }
    }

    pub fn pop_all(&mut self) -> &[UntypedMessage] {
        let count = self.count() as usize;
        if count == 0 {
            return &[];
        }

        unsafe {
            let start = count - 1;

            // Берем указатель на начало данных
            let start_ptr = self.slice.add(start).as_ptr();

            // Обнуляем счетчик
            self.count.write(0);

            core::slice::from_raw_parts(start_ptr, count)
        }
    }
}
