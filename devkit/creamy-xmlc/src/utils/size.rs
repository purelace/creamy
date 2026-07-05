use std::num::NonZeroU8;

use binrw::{BinRead, BinWrite};

use crate::{
    constraints::MAX_PAYLOAD,
    error::{Fallback, SemanticError},
};

/// This struct is guaranteed that size is non-zero and equal to or less than [`Self::MAX_VALUE`];
#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(NonZeroU8);
impl Size {
    #[allow(clippy::cast_possible_truncation)]
    pub const MAX_VALUE: u8 = MAX_PAYLOAD as u8;

    pub const B1: Self = Self(NonZeroU8::new(1).unwrap());
    pub const B2: Self = Self(NonZeroU8::new(2).unwrap());
    pub const B4: Self = Self(NonZeroU8::new(4).unwrap());
    pub const B8: Self = Self(NonZeroU8::new(8).unwrap());
    pub const B16: Self = Self(NonZeroU8::new(16).unwrap());

    pub const fn new(size: u8) -> Result<Self, SemanticError> {
        if size > Self::MAX_VALUE {
            return Err(SemanticError::InvalidSize {
                actual: size as usize,
            });
        }
        let Some(value) = NonZeroU8::new(size) else {
            return Err(SemanticError::InvalidSize { actual: 0 });
        };
        Ok(Self(value))
    }

    #[must_use]
    pub const fn value(self) -> u8 {
        self.0.get()
    }
}

impl Fallback for Size {
    fn fallback() -> Self {
        Self::B1
    }
}
