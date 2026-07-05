#![allow(clippy::as_conversions)]
#![allow(clippy::cast_possible_truncation)]

use std::ops::{Deref, Index, RangeBounds};

use binrw::{BinRead, BinWrite};

use crate::utils::Range;

#[derive(Debug, PartialEq, Eq)]
pub struct BoundedVec<T: VectorElement> {
    inner: Vec<T>,
}

impl<T: VectorElement> Default for BoundedVec<T> {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl<T: VectorElement> BoundedVec<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: vec![] }
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
    pub const fn len(&self) -> u32 {
        self.inner.len() as u32
    }

    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }

    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> std::vec::Drain<'_, T> {
        self.inner.drain(range)
    }

    //pub fn slice(&self, range: T::RangeType) -> &[T] {
    //    self[range]
    //}
}

//impl<T: VectorElement> DerefMut for BoundedVec<T> {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        &mut self.inner
//    }
//}
//
//impl<T: VectorElement> Deref for BoundedVec<T> {
//    type Target = Vec<T>;
//
//    fn deref(&self) -> &Self::Target {
//        &self.inner
//    }
//}

impl<T: BinRead + VectorElement> BinRead for BoundedVec<T>
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
        let mut inner = Vec::with_capacity(len);
        for _ in 0..len {
            inner.push(T::read_options(reader, endian, args)?);
        }

        Ok(Self { inner })
    }
}

impl<T: BinWrite + VectorElement> BinWrite for BoundedVec<T>
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
        let slice = &self.inner[..self.inner.len()];
        for item in slice {
            item.write_options(writer, endian, args)?;
        }
        Ok(())
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
