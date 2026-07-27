use alloc::boxed::Box;
use core::{any::Any, num::NonZeroU8};

use downcast_rs::{Downcast, impl_downcast};

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubscriberId(NonZeroU8);

impl SubscriberId {
    #[must_use]
    #[inline]
    pub const fn new(id: NonZeroU8) -> Self {
        Self(id)
    }

    #[must_use]
    #[inline]
    pub const fn new_u8(id: u8) -> Option<Self> {
        match NonZeroU8::new(id) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    #[inline]
    pub const unsafe fn new_unchecked(id: u8) -> Self {
        debug_assert!(id != 0);
        unsafe { Self(NonZeroU8::new_unchecked(id)) }
    }

    #[must_use]
    #[inline]
    pub const fn as_inner(self) -> NonZeroU8 {
        self.0
    }

    #[must_use]
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self.0.get()
    }

    #[must_use]
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0.get() as usize
    }
}

//impl Display for SubscriberId {
//    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//        write!(f, "{}", self.0)
//    }
//}

impl core::ops::Deref for SubscriberId {
    type Target = NonZeroU8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Subscriber: Downcast + Any + 'static {
    fn notify(&mut self);
}

impl Subscriber for () {
    fn notify(&mut self) {}
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
        #[derive(Debug)]
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
