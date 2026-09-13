#![cfg_attr(coverage_nightly, coverage(off))]

use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct List<T>(Vec<T>);

impl<T> Default for List<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<T> List<T> {
    #[must_use]
    pub const fn wrap(vec: Vec<T>) -> Self {
        Self(vec)
    }

    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn with_capacity(capacity: u32) -> Self {
        Self(Vec::with_capacity(capacity as usize))
    }
}

impl<T> Deref for List<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: binrw::BinRead> binrw::BinRead for List<T>
where
    T: for<'a> binrw::BinRead<Args<'a> = ()>,
{
    type Args<'a> = ();

    fn read_options<R: binrw::io::Read + binrw::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let len = u32::read_options(reader, endian, args)?;
        let mut buf = Vec::with_capacity(len as usize);
        for _ in 0..len {
            buf.push(T::read_options(reader, endian, args)?);
        }

        Ok(List(buf))
    }
}

impl<T: binrw::BinWrite> binrw::BinWrite for List<T>
where
    T: for<'a> binrw::BinWrite<Args<'a> = ()>,
{
    type Args<'a> = ();

    fn write_options<W: binrw::io::Write + binrw::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        (self.len() as u32).write_options(writer, endian, ())?;
        for s in self.iter() {
            s.write_options(writer, endian, args)?;
        }
        Ok(())
    }
}
