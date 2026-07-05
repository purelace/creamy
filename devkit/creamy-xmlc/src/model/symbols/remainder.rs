use std::num::NonZeroU16;

use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::NonZeroStringId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Remainder(Option<NonZeroStringId>);
impl Remainder {
    #[must_use]
    pub const fn new(value: Option<NonZeroStringId>) -> Self {
        Self(value)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use]
    pub fn into_inner(self) -> Option<NonZeroStringId> {
        self.0
    }
}

impl BinRead for Remainder {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let value = u16::read_options(reader, endian, args)?;
        if let Some(value) = NonZeroU16::new(value) {
            Ok(Self(Some(NonZeroStringId::new(value))))
        } else {
            Ok(Self(None))
        }
    }
}

impl BinWrite for Remainder {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        if let Some(value) = self.0 {
            value.write_options(writer, endian, args)
        } else {
            (0u16).write_options(writer, endian, args)
        }
    }
}
