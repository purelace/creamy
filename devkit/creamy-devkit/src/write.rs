use std::{io::BufWriter, path::Path};

use binrw::BinWrite;
use fs_err as fs;

use crate::BinaryPlugin;

impl BinaryPlugin {
    pub fn write_to_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = path.as_ref();
        let file = fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.write(&mut writer)?;
        Ok(())
    }
}
