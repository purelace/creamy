/// # Available values:
/// * 0 `[000]` - 1 byte,
/// * 1 `[001]` - 2 bytes,
/// * 2 `[010]` - 4 bytes,
/// * 3 `[011]` - 8 bytes,
/// * 4 `[100]` - 16 bytes,
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
//TODO check
pub enum Align {
    B1 = 0,
    B2 = 1,
    B4 = 2,
    B8 = 3,
    B16 = 4,
}

impl Align {
    #[must_use]
    pub const fn value(self) -> u8 {
        match self {
            Align::B1 => 1,
            Align::B2 => 2,
            Align::B4 => 4,
            Align::B8 => 8,
            Align::B16 => 16,
        }
    }

    #[must_use]
    pub const fn to_raw(value: u8) -> u8 {
        match value {
            1 => 0,
            2 => 1,
            4 => 2,
            8 => 3,
            16 => 4,
            _ => unimplemented!(),
        }
    }

    #[must_use]
    pub const fn raw_value(self) -> u8 {
        unsafe { std::mem::transmute(self) }
    }
}
