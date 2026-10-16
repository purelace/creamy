#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::{boxed::Box, string::String};

mod endianness;
mod impls;
mod length;

pub use embedded_io::{Read, Write};

pub use self::endianness::{BigEndian, Endianness, LittleEndian, NativeEndian};
use self::length::UnsignedInteger;

#[derive(Debug, thiserror::Error)]
pub struct CustomError {
    pub message: String,
    pub error: Option<Box<dyn core::error::Error>>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl core::fmt::Display for CustomError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if !self.message.is_empty() {
            write!(f, "{}", self.message)?;
        }

        if let Some(e) = self.error.as_ref() {
            write!(f, ": {e}")?;
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError<W: core::error::Error> {
    #[error("{0}")]
    Write(#[from] W),
    #[error("{0}")]
    Custom(CustomError),
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError<W: core::error::Error> {
    #[error("End of buffer")]
    EndOfBuffer,
    #[error("Incorrect length. Expected: {expected}, actual: {actual}")]
    IncorrectLength { expected: usize, actual: usize },
    #[error("value cannot be zero")]
    ZeroValue,
    #[error("{0}")]
    Read(#[from] W),
    #[error("{0}")]
    Custom(CustomError),
}

pub trait BinaryWriter {
    /// Writes the [`Self`] into the given writer using the specified endian byte order.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The underlying writer fails to write data.
    /// - Can't write the specified type.
    fn write<W: Write, E: Endianness>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>>;

    /// Writes the [`Self`] into the given writer using little endian byte order.
    ///
    /// # Errors
    ///
    /// See [`write`] method.
    fn write_le<W: Write>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>> {
        self.write::<W, LittleEndian>(writer)
    }

    /// Writes the [`Self`] into the given writer using big endian byte order.
    ///
    /// # Errors
    ///
    /// See [`write`] method.
    fn write_be<W: Write>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>> {
        self.write::<W, BigEndian>(writer)
    }

    /// Writes the [`Self`] into the given writer using native endian byte order.
    ///
    /// # Errors
    ///
    /// See [`write`] method.
    fn write_ne<W: Write>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>> {
        self.write::<W, NativeEndian>(writer)
    }
}

pub trait BinaryReader: Sized {
    /// Reads the [`Self`] from the given reader using the specified endian byte order.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The reader is empty.
    /// - An incorrect length is read for an array `[T; N]` (N < actual).
    /// - The underlying reader fails to read data.
    /// - Can't parse the specified type.
    fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>>;

    /// Reads the [`Self`] from the given reader using little endian byte order.
    ///
    /// # Errors
    /// See [`read`] method.
    fn read_le<R: Read>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        Self::read::<R, LittleEndian>(reader)
    }

    /// Reads the [`Self`] from the given reader using big endian byte order.
    ///
    /// # Errors
    /// See [`read`] method
    fn read_be<R: Read>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        Self::read::<R, BigEndian>(reader)
    }

    /// Reads the [`Self`] from the given reader using native endian byte order.
    ///
    /// # Errors
    /// See [`read`] method.
    fn read_ne<R: Read>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        Self::read::<R, NativeEndian>(reader)
    }
}

pub trait Buffer {
    type Item;

    type Iter<'a>: IntoIterator<Item = &'a Self::Item>
    where
        Self: 'a;

    type Length: UnsignedInteger;
    fn length(&self) -> Self::Length;
    fn iterator(&self) -> Self::Iter<'_>;
}

pub trait BufferWriter: Buffer<Item: BinaryWriter> + BinaryWriter {}
impl<T> BufferWriter for T
where
    T: Buffer + ?Sized,
    T::Item: BinaryWriter,
{
}

pub trait BufferReader: Buffer<Item: BinaryReader> + Sized {
    type Temp: TempBuffer<Base = Self>;
    /// Creates the instance of [`Self`] from temp buffer.
    ///
    /// # Errors
    ///
    /// Returns an error when the convertation fails.
    fn from_temp(
        buffer: <<Self as BufferReader>::Temp as TempBuffer>::Type,
    ) -> Result<Self, CustomError>;
}

pub trait TempBuffer {
    type Type;
    type Base: BufferReader;

    fn add(
        buffer: &mut Self::Type,
        index: <<Self as TempBuffer>::Base as Buffer>::Length,
        item: <<Self as TempBuffer>::Base as Buffer>::Item,
    );

    fn new_with_capacity(capacity: <<Self as TempBuffer>::Base as Buffer>::Length) -> Self::Type;
}

impl<T: BufferWriter + ?Sized> BinaryWriter for T {
    fn write<W: Write, E: Endianness>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>> {
        self.length().write::<W, E>(writer)?;
        for element in self.iterator() {
            element.write::<W, E>(writer)?;
        }
        Ok(())
    }
}

impl<T: BufferReader> BinaryReader for T {
    fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        let length = T::Length::read::<R, E>(reader)?;
        let mut buffer = T::Temp::new_with_capacity(length);
        for index in length.range_from_zero() {
            let item = T::Item::read::<R, E>(reader)?;
            T::Temp::add(&mut buffer, index, item);
        }

        T::from_temp(buffer).map_err(ReadError::Custom)
    }
}
