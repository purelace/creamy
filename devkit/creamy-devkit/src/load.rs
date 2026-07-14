use std::{io::Cursor, path::Path};

use binrw::BinRead;
use fs_err as fs;

use crate::BinaryPlugin;

impl BinaryPlugin {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let mut file = fs::File::open(path)?;
        Ok(Self::read(&mut file)?)
    }

    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut reader = Cursor::new(bytes);
        Ok(Self::read(&mut reader)?)
    }
}
