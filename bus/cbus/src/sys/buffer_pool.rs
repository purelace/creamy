use std::{alloc::Layout, marker::PhantomData, ptr::NonNull};

use as_guard::AsGuard;

use crate::{
    core::{
        UntypedMessage,
        buffer::{Buffer, Incoming, Outgoing, Read, Write},
    },
    defines::{MESSAGE_SIZE, METADATA, SPECIAL_DATA_OFFSET},
    sys::layout::alloc_pool,
};

#[repr(C, align(64))]
pub struct Header {
    pub count: u32,
    _padding: [u8; 60],
}

impl Header {
    pub const fn write_raw_mut_ptr(ptr: *mut Self) -> *mut UntypedMessage {
        unsafe {
            let count = (*ptr).count as usize;
            let data_ptr = ptr.add(1) as *mut u8;
            let write_ptr = data_ptr.add(count * MESSAGE_SIZE);
            write_ptr.cast::<UntypedMessage>()
        }
    }

    pub const fn read_slice_mut_test(
        ptr: *mut Header,
        capacity: usize,
    ) -> &'static mut [UntypedMessage] {
        let count = unsafe { (*ptr).count as usize };

        unsafe {
            // Считаем начало данных: адрес header + 1 (размер Header)
            let data_start = ptr.add(1) as *mut u8;

            let slice_start = data_start
                .add((capacity - count - 1) * MESSAGE_SIZE)
                .cast::<UntypedMessage>();

            // Создаем слайс только в самом конце
            std::slice::from_raw_parts_mut(slice_start, count)
        }
    }

    pub const fn set_count(ptr: *mut Header, count: usize) {
        unsafe {
            (*ptr).count = count as u32;
        }
    }

    pub const fn write_slice_mut(ptr: *mut Self, count: usize) -> &'static mut [UntypedMessage] {
        unsafe {
            let ptr = Header::write_raw_mut_ptr(ptr);
            std::slice::from_raw_parts_mut(ptr, count)
        }
    }
}

pub struct BufferPool<O> {
    layout: Layout,
    bytes: NonNull<u8>,
    slice_size: usize,
    capacity: usize,

    _operation: PhantomData<O>,
}

impl<O> Drop for BufferPool<O> {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.bytes.as_ptr(), self.layout);
        }
    }
}

impl<O> BufferPool<O> {
    pub fn new(slices: usize, max_messages: usize) -> Self {
        let slice_size = max_messages * MESSAGE_SIZE;
        let slice_size = slice_size + METADATA;
        let total_size = slice_size * slices;

        let (layout, bytes) = alloc_pool(total_size);
        Self {
            layout,
            bytes,
            capacity: slices,
            slice_size,
            _operation: PhantomData,
        }
    }

    pub const fn header_mut_ptr_for(&mut self, dst: usize) -> *mut Header {
        let start = self.u_mut_slice_for(dst);
        start.cast::<Header>().as_ptr()
    }

    #[inline(always)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn u_count_for(&self, src: usize) -> usize {
        unsafe {
            // В начале каждого отрезка памяти находится счетчик сообщений (4 байта + 60 байт отступ)
            self.u_slice_for(src).cast::<u32>().read() as usize
        }
    }

    /// Возвращает указатель на начало слайса, привязанного к конкретному подписчику
    /// Принимает dst как usize
    #[inline(always)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn u_slice_for(&self, src: usize) -> NonNull<u8> {
        unsafe {
            // Считаем глобальный адрес нужного отрезка памяти.
            self.bytes.add(src * self.slice_size)
        }
    }

    /// Возвращает изменяемый указатель на начало слайса, привязанного к конкретному подписчику
    /// Принимает dst как usize
    #[inline(always)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn u_mut_slice_for(&mut self, src: usize) -> NonNull<u8> {
        unsafe {
            // Считаем глобальный адрес нужного отрезка памяти.
            self.bytes.add(src * self.slice_size)
        }
    }

    #[inline(always)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn slice_capacity(&self) -> usize {
        (self.slice_size - SPECIAL_DATA_OFFSET) / MESSAGE_SIZE
    }

    // TODO
    /// Returns the next slice of the fixed pool.
    ///
    /// # Returns
    ///
    /// * `Some(Incoming)` - The next slice of the fixed pool.
    /// * `None` - If the fixed pool is full.
    fn internal_next_slice(&mut self, id: usize) -> Option<Buffer<O>> {
        if id >= self.capacity {
            return None;
        }

        let start = id * self.slice_size;

        unsafe {
            let capacity = ((self.slice_size - SPECIAL_DATA_OFFSET) / MESSAGE_SIZE).safe_as();
            let count = self.bytes.add(start).cast::<u32>();
            let buffer = self.bytes.add(start + METADATA).cast::<UntypedMessage>();

            // Инициализируем переменные, иначе пиздец будет
            // Reset count
            count.write(0);

            // Reset ignore flag
            count.add(1).write(0);

            Some(Buffer::new(count, buffer, capacity))
        }
    }

    pub const fn return_buffer(&mut self, id: usize) {
        let start = id * self.slice_size;
        unsafe {
            let count = self.bytes.add(start).cast::<u32>();
            count.write(0);
        }
    }
}

impl BufferPool<Read> {
    pub fn next_inc(&mut self, id: u8) -> Option<Incoming> {
        self.internal_next_slice(id as usize)
    }
}

impl BufferPool<Write> {
    pub fn next_out(&mut self, id: u8) -> Option<Outgoing> {
        self.internal_next_slice(id as usize)
    }
}
