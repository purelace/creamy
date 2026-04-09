use std::{alloc::Layout, marker::PhantomData, ptr::NonNull};

use as_guard::AsGuard;
use idmint::HeapMint;

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
    const fn data(&self) -> NonNull<u8> {
        unsafe {
            // self + 1 пропустит ровно sizeof(Header), т.е. 64 байта
            NonNull::new_unchecked((self as *const Header).add(1) as *mut u8)
        }
    }

    #[inline(always)]
    pub const fn write_ptr(&self) -> NonNull<UntypedMessage> {
        unsafe {
            self.data()
                .add(self.count as usize * MESSAGE_SIZE)
                .cast::<UntypedMessage>()
        }
    }

    pub const fn read_ptr(&self, capacity: usize) -> NonNull<UntypedMessage> {
        unsafe {
            // Перемещаем указатель на последнее сообщение в буфере (отсчет идет с конца)
            self.data()
                .add((capacity - self.count as usize - 1) * MESSAGE_SIZE)
                .cast::<UntypedMessage>()
        }
    }

    pub const fn read_slice(&self, capacity: usize) -> &[UntypedMessage] {
        unsafe {
            let read_ptr = self.read_ptr(capacity);
            std::slice::from_raw_parts(read_ptr.as_ptr(), self.count as usize)
        }
    }

    pub const fn read_slice_mut(&mut self, capacity: usize) -> &mut [UntypedMessage] {
        unsafe {
            let slice_start = self
                .data()
                .add((capacity - self.count as usize - 1) * MESSAGE_SIZE)
                .cast::<UntypedMessage>()
                .as_ptr();

            std::slice::from_raw_parts_mut(slice_start, self.count as usize)
        }
    }

    pub const fn write_slice_mut(&mut self, count: usize) -> &mut [UntypedMessage] {
        unsafe {
            let ptr = self.write_ptr().as_ptr();
            std::slice::from_raw_parts_mut(ptr, count)
        }
    }
}

pub struct BufferPool<O> {
    layout: Layout,
    bytes: NonNull<u8>,
    slice_size: usize,
    capacity: usize,

    provider: HeapMint<usize>,
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
            provider: HeapMint::default(),
            _operation: PhantomData,
        }
    }

    pub const fn header_for(&mut self, src: usize) -> &mut Header {
        unsafe {
            let start = self.u_slice_for(src);
            &mut *start.as_ptr().cast::<Header>()
        }
    }

    pub const fn header_ptr_for(&self, dst: usize) -> NonNull<Header> {
        let start = self.u_slice_for(dst);
        start.cast::<Header>()
    }

    pub fn return_buffer(&mut self, index: usize) {
        self.provider.recycle(index);
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
    fn internal_next_slice(&mut self) -> Option<Buffer<O>> {
        let used = self.provider.used();
        if used >= self.capacity {
            return None;
        }

        let next_used = self.provider.issue().unwrap();
        let used = if next_used < used {
            next_used
        } else {
            self.provider.recycle(next_used);
            used
        };

        let start = used * self.slice_size;

        unsafe {
            let capacity = ((self.slice_size - SPECIAL_DATA_OFFSET) / MESSAGE_SIZE).safe_as();
            let count = self.bytes.add(start).cast::<u32>();
            let buffer = self.bytes.add(start + METADATA).cast::<UntypedMessage>();

            // Инициализируем переменные, иначе пиздец будет
            // Reset count
            count.write(0);

            // Reset ignore flag
            count.add(1).write(0);

            let _ = self.provider.issue();

            Some(Buffer::new(count, buffer, capacity))
        }
    }
}

impl BufferPool<Read> {
    pub fn next_inc(&mut self) -> Option<Incoming> {
        self.internal_next_slice()
    }
}

impl BufferPool<Write> {
    pub fn next_out(&mut self) -> Option<Outgoing> {
        self.internal_next_slice()
    }
}
