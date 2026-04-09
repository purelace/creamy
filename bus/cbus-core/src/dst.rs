#![cfg_attr(coverage_nightly, coverage(off))]

use core::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use bytemuck::{Pod, Zeroable};

use crate::subscriber::SubscriberId;

#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroable, Pod)]
pub struct Destination(u8);

impl Display for Destination {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Destination {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Destination {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Destination {
    pub const ZERO: Self = Self(0);

    #[must_use]
    #[inline(always)]
    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    #[must_use]
    #[inline(always)]
    pub const fn as_subscriber_id(self) -> SubscriberId {
        SubscriberId::new(self.0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn cast_usize(self) -> usize {
        self.0 as usize
    }

    #[must_use]
    #[inline(always)]
    pub const fn u8(self) -> u8 {
        self.0
    }
}
