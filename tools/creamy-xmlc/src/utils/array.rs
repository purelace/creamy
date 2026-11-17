use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Array<T> {
    inner: Box<[T]>,
}

impl<T> Array<T> {
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        &self.inner
    }
}

impl<T> Array<MaybeUninit<T>> {
    #[must_use]
    pub fn zeroed(size: usize) -> Self {
        Self {
            inner: core::iter::repeat_with(|| MaybeUninit::zeroed())
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
}

impl<T> From<Box<[T]>> for Array<T> {
    fn from(value: Box<[T]>) -> Self {
        Self { inner: value }
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
