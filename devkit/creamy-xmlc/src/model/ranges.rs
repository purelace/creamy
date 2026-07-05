use std::fmt::Display;

use binrw::{BinRead, BinWrite};

use crate::define_readonly_struct;

/// Max offset value: 2^27
/// Max length value: 28
#[derive(BinRead, BinWrite, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangePackedU32 {
    /// Packed value
    /// 00000 - len
    /// `00000000_00000000_00000000_000` - start
    value: u32,
}

impl Display for RangePackedU32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.start();
        let len = self.len();
        write!(f, "{start}..{len}")
    }
}

impl RangePackedU32 {
    #[must_use]
    pub const fn new(start: u32, len: u8) -> Self {
        assert!(start <= 2u32.pow(27));
        assert!(len <= 28);
        let len = (len as u32) << 27;
        Self { value: start | len }
    }

    #[must_use]
    pub const fn start(&self) -> u32 {
        self.value & 0b00000111_11111111_11111111_11111111
    }

    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub const fn len(&self) -> u8 {
        (self.value >> 27) as u8
    }
}

pub type Structs = RangePackedU32;
pub type Fields = RangePackedU32;
pub type Enums = RangePackedU32;
pub type Flags = RangePackedU32;
pub type Bitsets = RangePackedU32;

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Messages {
    start: u8,
    len: u8,
}

impl Messages {
    #[must_use]
    pub const fn new(start: u8, len: u8) -> Self {
        //TODO: overflow checks
        Self { start, len }
    }

    #[must_use]
    pub const fn start(&self) -> u8 {
        self.start
    }

    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub const fn len(&self) -> u8 {
        self.len
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variants {
    start: u16,
    len: u16,
}

impl Variants {
    #[must_use]
    pub const fn new(start: u16, len: u16) -> Self {
        Self { start, len }
    }

    #[must_use]
    pub const fn start(&self) -> u16 {
        self.start
    }

    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub const fn len(&self) -> u16 {
        self.len
    }
}

define_readonly_struct! {
    struct Options {
        start: u16,
        len: u16,
    }
}

define_readonly_struct! {
    struct BitsetValues {
        start: u16,
        len: u16,
    }
}
