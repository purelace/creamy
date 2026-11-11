#![cfg_attr(coverage_nightly, coverage(off))]
use core::num::NonZeroU8;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupId(NonZeroU8);

impl GroupId {
    #[must_use]
    pub const fn new(id: NonZeroU8) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn new_u8(id: u8) -> Option<Self> {
        match NonZeroU8::new(id) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const unsafe fn new_unchecked(id: u8) -> Self {
        debug_assert!(id != 0);
        unsafe { Self(NonZeroU8::new_unchecked(id)) }
    }

    #[must_use]
    pub const fn as_non_zero_u8(self) -> NonZeroU8 {
        self.0
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0.get()
    }

    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0.get() as usize
    }
}

impl core::fmt::Display for GroupId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
