use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use binrw::{BinRead, BinWrite};

#[derive(Debug, PartialEq, Eq)]
pub struct Array<T> {
    inner: Box<[T]>,
}

impl<T> Array<T> {
    /// # Panics
    ///
    /// Panics if ``count`` argument is more than array length.
    pub fn trim(&mut self, count: usize) {
        let len = self.inner.len();
        if len == count {
            return;
        }

        assert!(len > count, "Count cannot be more than array length");

        let uninit = unsafe { Array::<MaybeUninit<T>>::zeroed(count).assume_init() };
        let old = std::mem::replace(&mut self.inner, uninit.inner);

        unsafe {
            std::ptr::copy_nonoverlapping(
                old.as_ptr(),            // Источник
                self.inner.as_mut_ptr(), // Назначение (новый массив)
                count,                   // Количество элементов для перемещения
            );
        }
    }
}

impl<T: Copy> Array<T> {
    pub fn new_with_default(size: usize, default: T) -> Self {
        Self {
            inner: std::iter::repeat_with(|| default).take(size).collect(),
        }
    }
}

impl<T> Array<MaybeUninit<T>> {
    #[must_use]
    pub fn zeroed(size: usize) -> Self {
        Self {
            inner: std::iter::repeat_with(|| MaybeUninit::zeroed())
                .take(size)
                .collect(),
        }
    }

    /// Converts an `Array<MaybeUninit<T>>` into an `Array<T>` by assuming
    /// all elements are fully initialized.
    ///
    /// This is a zero-cost operation as `MaybeUninit<T>` is guaranteed to have
    /// the same memory layout as `T`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that every element in the array has been
    /// fully initialized.
    ///
    /// Failure to fulfill this requirement results in immediate Undefined Behavior.
    #[must_use]
    pub unsafe fn assume_init(self) -> Array<T> {
        let ptr = Box::into_raw(self.inner) as *mut [T];
        let inner = unsafe { Box::from_raw(ptr) };
        Array { inner }
    }

    /// Returns a reference to the underlying data as a slice of initialized elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. All elements in the array have been fully initialized.
    /// 2. The memory layout of `T` is compatible with the initialized data.
    ///
    /// Accessing uninitialized elements through the resulting slice is Undefined Behavior.
    /// Since this returns a reference, the original `Array` must outlive the slice,
    /// and the data must not be modified in a way that invalidates `T` while the slice exists.
    #[must_use]
    pub unsafe fn assume_init_ref_slice(&self) -> &[T] {
        let ptr = self.inner.as_ptr().cast::<T>();
        let len = self.inner.len();
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

impl<T> From<Box<[T]>> for Array<T> {
    fn from(value: Box<[T]>) -> Self {
        Self { inner: value }
    }
}

impl<T: BinRead> BinRead for Array<T>
where
    T: for<'a> BinRead<Args<'a> = ()>,
{
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let len = u32::read_options(reader, endian, args)? as usize;
        let mut array = Array::<MaybeUninit<T>>::zeroed(len);
        for index in 0..len {
            array.inner[index] = MaybeUninit::new(T::read_options(reader, endian, args)?);
        }

        unsafe { Ok(array.assume_init()) }
    }
}

impl<T: BinWrite> BinWrite for Array<T>
where
    T: for<'a> BinWrite<Args<'a> = ()>,
{
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        (self.inner.len() as u32).write_options(writer, endian, args)?;
        for s in &self.inner[..self.inner.len()] {
            s.write_options(writer, endian, args)?;
        }
        Ok(())
    }
}

impl<T> Deref for Array<T> {
    type Target = Box<[T]>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
