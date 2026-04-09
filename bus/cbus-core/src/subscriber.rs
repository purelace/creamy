use alloc::boxed::Box;
use core::{
    any::Any,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use bytemuck::{Pod, Zeroable};
use downcast_rs::{Downcast, impl_downcast};

use crate::Destination;

#[repr(transparent)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroable, Pod)]
pub struct SubscriberId(u8);

impl SubscriberId {
    pub const ZERO: Self = Self(0);

    #[must_use]
    #[inline(always)]
    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    #[must_use]
    #[inline(always)]
    pub const fn u8(self) -> u8 {
        self.0
    }

    #[must_use]
    #[inline(always)]
    pub const fn cast_usize(self) -> usize {
        self.0 as usize
    }

    #[must_use]
    #[inline(always)]
    pub const fn as_dst(self) -> Destination {
        Destination::new(self.0)
    }
}

impl Display for SubscriberId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for SubscriberId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SubscriberId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait Subscriber: Downcast + Any + 'static {
    fn notify(&mut self);
}

impl<S: Subscriber + ?Sized> Subscriber for Box<S> {
    fn notify(&mut self) {
        (**self).notify();
    }
}

#[macro_export]
macro_rules! subscribers {
    ($name:ident,
        $(
            $variant:ident => $ty:ty,
        )*
    ) => {
        pub enum $name {
            $(
                $variant($ty),
            )*
        }

        impl Subscriber for $name {
            fn notify(&mut self) {
                match self {
                    $(
                        $name::$variant(value) => value.notify(),
                    )*
                }
            }
        }

        $(
            impl From<$ty> for $name {
                fn from(value: $ty) -> Self {
                    $name::$variant(value)
                }
            }
        )*
    };
}

impl_downcast!(Subscriber);
