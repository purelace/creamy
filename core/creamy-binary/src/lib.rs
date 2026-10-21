#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::{boxed::Box, string::String};

mod dyn_support;
mod endianness;
mod impls;

pub use creamy_binary_derive::{BinRead, BinWrite};
pub use dyn_support::*;
pub use embedded_io::{ErrorType, Read, Write};

pub use self::endianness::{BigEndian, Endianness, LittleEndian, NativeEndian};

#[cfg(feature = "std")]
pub mod adapters {
    pub use embedded_io_adapters::std::*;
}

#[derive(Debug, thiserror::Error)]
pub struct CustomError {
    pub message: String,
    pub error: Option<Box<dyn core::error::Error + Send + Sync + 'static>>,
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
pub enum WriteError<W: core::error::Error + Send + Sync + 'static> {
    #[error("{0}")]
    Write(#[from] W),
    #[error("{0}")]
    Custom(CustomError),
    #[error("{0}")]
    Dyn(DynError),
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
    fn write<W: Write<Error: Send + Sync + 'static>, E: Endianness>(
        &self,
        writer: &mut W,
    ) -> Result<(), WriteError<W::Error>>;

    /// Writes the [`Self`] into the given writer using little endian byte order.
    ///
    /// # Errors
    ///
    /// See [`write`] method.
    fn write_le<W: Write<Error: Send + Sync + 'static>>(
        &self,
        writer: &mut W,
    ) -> Result<(), WriteError<W::Error>> {
        self.write::<W, LittleEndian>(writer)
    }

    /// Writes the [`Self`] into the given writer using big endian byte order.
    ///
    /// # Errors
    ///
    /// See [`write`] method.
    fn write_be<W: Write<Error: Send + Sync + 'static>>(
        &self,
        writer: &mut W,
    ) -> Result<(), WriteError<W::Error>> {
        self.write::<W, BigEndian>(writer)
    }

    /// Writes the [`Self`] into the given writer using native endian byte order.
    ///
    /// # Errors
    ///
    /// See [`write`] method.
    fn write_ne<W: Write<Error: Send + Sync + 'static>>(
        &self,
        writer: &mut W,
    ) -> Result<(), WriteError<W::Error>> {
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
