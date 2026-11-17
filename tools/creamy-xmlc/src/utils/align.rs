use self::Align::{B1, B2, B4, B8, B16};

/// # Available values:
/// * 0 `[000]` - 1 byte,
/// * 1 `[001]` - 2 bytes,
/// * 2 `[010]` - 4 bytes,
/// * 3 `[011]` - 8 bytes,
/// * 4 `[100]` - 16 bytes,
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    B1 = 0,
    B2 = 1,
    B4 = 2,
    B8 = 3,
    B16 = 4,
}

impl Align {
    #[must_use]
    pub const fn size(self) -> u64 {
        match self {
            Align::B1 => 1,
            Align::B2 => 2,
            Align::B4 => 4,
            Align::B8 => 8,
            Align::B16 => 16,
        }
    }

    #[must_use]
    pub const fn new(value: u64) -> Option<Self> {
        match value {
            1 => Some(B1),
            2 => Some(B2),
            4 => Some(B4),
            8 => Some(B8),
            16 => Some(B16),
            _ => None,
        }
    }
}
