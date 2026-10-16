use alloc::boxed::Box;

use embedded_io::{ErrorType, Write};

use crate::{BinaryWriter, WriteError};

pub struct DynError(Box<dyn embedded_io::Error + Send + Sync + 'static>);
impl core::fmt::Debug for DynError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl core::fmt::Display for DynError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl core::error::Error for DynError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.0.source()
    }

    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.0.description()
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn core::error::Error> {
        self.0.cause()
    }
}

impl embedded_io::Error for DynError {
    fn kind(&self) -> embedded_io::ErrorKind {
        self.0.kind()
    }
}

pub struct WriterInstance<'a, W>
where
    W: Write,
    W::Error: core::error::Error + Send + Sync + 'static,
{
    writer: &'a mut W,
}

impl<W> ErrorType for WriterInstance<'_, W>
where
    W: Write,
    W::Error: core::error::Error + Send + Sync + 'static,
{
    type Error = DynError;
}

impl<W> Write for WriterInstance<'_, W>
where
    W: Write,
    W::Error: core::error::Error + Send + Sync + 'static,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.writer.write(buf).map_err(|e| DynError(Box::new(e)))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.writer.flush().map_err(|e| DynError(Box::new(e)))
    }
}

pub trait DynWrite<W>
where
    W: Write,
    W::Error: core::error::Error + Send + Sync + 'static,
{
    fn as_dyn_writer(&mut self) -> WriterInstance<'_, W>;
}

impl<W> DynWrite<W> for W
where
    W: Write,
    W::Error: core::error::Error + Send + Sync + 'static,
{
    fn as_dyn_writer(&mut self) -> WriterInstance<'_, W> {
        WriterInstance { writer: self }
    }
}

pub struct DynWriter<'a>(&'a mut dyn Write<Error = DynError>);
impl<'a> DynWriter<'a> {
    pub const fn new<T: Write<Error = DynError>>(writer: &'a mut T) -> Self {
        Self(writer)
    }
}

impl embedded_io::ErrorType for DynWriter<'_> {
    type Error = DynError;
}

impl Write for DynWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }
}

#[derive(Clone, Copy)]
pub enum DynEndianness {
    Little,
    Big,
    Native,
}

pub trait DynBinaryWrite {
    /// Writes the [`Self`] into the dyn writer using the specified endian byte order.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Writer fails to write data.
    /// - Can't write the specified type.
    fn dyn_write(
        &self,
        endian: DynEndianness,
        writer: &mut DynWriter,
    ) -> Result<(), WriteError<DynError>>;

    /// Writes the [`Self`] into the dyn writer using little endian byte order.
    ///
    /// # Errors
    ///
    /// See [`dyn_write`] method.
    fn dyn_write_le(&self, writer: &mut DynWriter) -> Result<(), WriteError<DynError>> {
        self.dyn_write(DynEndianness::Little, writer)
    }

    /// Writes the [`Self`] into the dyn writer using big endian byte order.
    ///
    /// # Errors
    ///
    /// See [`dyn_write`] method.
    fn dyn_write_be(&self, writer: &mut DynWriter) -> Result<(), WriteError<DynError>> {
        self.dyn_write(DynEndianness::Big, writer)
    }

    /// Writes the [`Self`] into the dyn writer using native endian byte order.
    ///
    /// # Errors
    ///
    /// See [`dyn_write`] method.
    fn dyn_write_ne(&self, writer: &mut DynWriter) -> Result<(), WriteError<DynError>> {
        self.dyn_write(DynEndianness::Native, writer)
    }
}

impl<T: BinaryWriter> DynBinaryWrite for T {
    fn dyn_write(
        &self,
        endian: DynEndianness,
        writer: &mut DynWriter,
    ) -> Result<(), WriteError<DynError>> {
        match endian {
            DynEndianness::Little => self.write_le(writer),
            DynEndianness::Big => self.write_be(writer),
            DynEndianness::Native => self.write_ne(writer),
        }
    }
}
