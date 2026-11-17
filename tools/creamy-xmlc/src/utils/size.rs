use core::num::NonZeroU64;

use crate::{
    MAX_PAYLOAD,
    error::{Fallback, SemanticError},
};

/// This struct is guaranteed that size is non-zero and equal to or less than [`Self::MAX_VALUE`];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// TODO: validate struct after deserialization
pub struct Size(NonZeroU64);
impl Size {
    #[allow(clippy::cast_possible_truncation)]
    pub const MAX_VALUE: u8 = MAX_PAYLOAD as u8;

    pub const B1: Self = Self(NonZeroU64::new(1).unwrap());
    pub const B2: Self = Self(NonZeroU64::new(2).unwrap());
    pub const B4: Self = Self(NonZeroU64::new(4).unwrap());
    pub const B8: Self = Self(NonZeroU64::new(8).unwrap());
    pub const B16: Self = Self(NonZeroU64::new(16).unwrap());

    /// Tries to wrap a raw size value into a safe object
    ///
    /// # Errors
    ///
    /// This function will return an error if `size == 0`
    pub const fn try_new(size: u64) -> Result<Self, SemanticError> {
        let Some(value) = NonZeroU64::new(size) else {
            return Err(SemanticError::InvalidSize { actual: 0 });
        };

        Ok(Self(value))
    }

    pub(crate) const unsafe fn new_unchecked(size: u64) -> Self {
        unsafe { Self(NonZeroU64::new_unchecked(size)) }
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0.get()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Fallback for Size {
    fn fallback() -> Self {
        Self::B1
    }
}
