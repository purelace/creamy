use std::ops::Deref;

use binrw::{BinRead, BinWrite};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BString(String);
impl Deref for BString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BinRead for BString {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let len = u32::read_options(reader, endian, ())?;
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;
        Ok(BString(String::from_utf8(buf).map_err(|e| {
            binrw::Error::Custom {
                pos: reader.stream_position().unwrap_or(0),
                err: Box::new(e),
            }
        })?))
    }
}

impl BinWrite for BString {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        (self.len() as u32).write_options(writer, endian, ())?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
}
