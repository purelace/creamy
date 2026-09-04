use std::{io::Cursor, path::Path};

use binrw::BinRead;
use fs_err as fs;

use crate::{BinaryPlugin, error::Error};

impl BinaryPlugin {
    /// Loads a `BinaryPlugin` from a file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a path pointing to a valid `.cmy` file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The file cannot be opened or read from the specified path.
    /// * The binary format is invalid.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let mut file = fs::File::open(path)?;
        Ok(Self::read(&mut file)?)
    }

    /// Loads a `BinaryPlugin` from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A slice containing the raw bytes of the binary plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the binary format is invalid.
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut reader = Cursor::new(bytes);
        Ok(Self::read(&mut reader)?)
    }
}
