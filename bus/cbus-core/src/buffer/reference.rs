use core::ptr::NonNull;

use super::shared::get_trashcan_ptr;
use crate::UntypedMessage;

pub struct RawBuf {
    pub(crate) count: NonNull<u32>,
    pub(crate) data: NonNull<UntypedMessage>,
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
    pub(crate) count: &'a mut u32,
    pub(crate) data: &'a mut [UntypedMessage],
}

impl<const M: usize> RefMutBuf<'_, M> {
    #[must_use]
    pub fn is_null(&self) -> bool {
        let ptr = (&raw const self.count).cast::<u8>();
        ptr == get_trashcan_ptr().as_ptr()
    }

    /// Очищает буфер
    pub const fn clear(&mut self) {
        *self.count = 0;
    }

    /// Добавляет указанное количество элементов
    ///
    /// # Safety
    ///
    /// Вызывающий код должен гарантировать, что результат не превышает `M`.
    pub const unsafe fn add_count(&mut self, count: u32) {
        debug_assert!(
            *self.count as usize + count as usize <= M,
            "new count exceeds buffer capacity"
        );
        *self.count = *self.count + count;
    }

    /// Возвращает количество сообщений
    #[must_use]
    pub const fn count(&self) -> u32 {
        *self.count
    }
}

impl<const M: usize> RefMutBuf<'_, M> {
    #[must_use]
    pub fn read_slice(&self) -> &[UntypedMessage] {
        let count = *self.count as usize;
        &self.data[M - count..M]
    }

    pub fn read_slice_mut(&mut self) -> &mut [UntypedMessage] {
        let count = *self.count as usize;
        &mut self.data[M - count..M]
    }

    //TODO: non zero u32
    pub fn write_slice_mut(&mut self, slice_size: u32) -> &mut [UntypedMessage] {
        let count = *self.count as usize;
        &mut self.data[count..count + slice_size as usize]
    }
}
