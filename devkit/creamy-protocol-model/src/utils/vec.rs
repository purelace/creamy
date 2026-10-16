#![allow(clippy::as_conversions)]

use alloc::vec::Vec;
use core::ops::{Deref, DerefMut, Index, RangeBounds};

use creamy_binary::{BinaryReader, BinaryWriter};

use crate::utils::Range;

//TODO: check length after deserialization
#[derive(Debug, PartialEq, Eq)]
pub struct BoundedVec<T: VectorElement> {
    inner: Vec<T>,
}

impl<T: BinaryReader + VectorElement> BinaryReader for BoundedVec<T> {
    fn read<R: creamy_binary::Read, E: creamy_binary::Endianness>(
        reader: &mut R,
    ) -> Result<Self, creamy_binary::ReadError<R::Error>> {
        Ok(Self {
            inner: Vec::read::<R, E>(reader)?,
        })
    }
}

impl<T: BinaryWriter + VectorElement> BinaryWriter for BoundedVec<T> {
    fn write<
        W: creamy_binary::Write<Error: Send + Sync + 'static>,
        E: creamy_binary::Endianness,
    >(
        &self,
        writer: &mut W,
    ) -> Result<(), creamy_binary::WriteError<W::Error>> {
        self.inner.write::<W, E>(writer)
    }
}

impl<T: VectorElement> Default for BoundedVec<T> {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl<T: VectorElement> BoundedVec<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: alloc::vec![],
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: u32) -> Self {
        assert!(
            capacity as usize <= T::MAX_SIZE,
            "Passed capacity: {capacity}, MAX_SIZE: {}",
            T::MAX_SIZE
        );
        Self {
            inner: Vec::with_capacity(capacity as usize),
        }
    }

    /// # Returns
    /// Возвращает ``true`` если элемент успешно добавлен, иначе ``false``
    #[must_use]
    pub fn push(&mut self, item: T) -> bool {
        if self.inner.len() >= T::MAX_SIZE {
            return false;
        }

        self.inner.push(item);
        true
    }

    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> u32 {
        self.inner.len() as u32
    }

    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }

    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> alloc::vec::Drain<'_, T> {
        self.inner.drain(range)
    }
}

pub trait VectorElement {
    const MAX_SIZE: usize;
    type RangeType: Range;
}

impl<T: VectorElement> Index<T::RangeType> for BoundedVec<T> {
    type Output = [T];

    fn index(&self, index: T::RangeType) -> &Self::Output {
        &self.inner[index.as_range()]
    }
}

impl<T: VectorElement> Deref for BoundedVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: VectorElement> DerefMut for BoundedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
