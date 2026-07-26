use core::num::{NonZeroU8, NonZeroU32};

#[derive(Clone, Copy)]
pub struct ValidU8(NonZeroU8);
impl ValidU8 {
    pub const MAX: Self = Self(NonZeroU8::new(u8::MAX - 1).unwrap());
    pub const ONE: Self = Self(NonZeroU8::new(1).unwrap());

    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value != 0 && value.is_multiple_of(2) {
            // SAFETY: The `value` is explicitly checked to be non-zero in the conditional expression above.
            // This strictly guarantees that the precondition for `NonZeroU8::new_unchecked` is met.
            unsafe {
                return Some(Self(NonZeroU8::new_unchecked(value)));
            }
        }

        None
    }

    #[must_use]
    pub const fn get(&self) -> u8 {
        self.0.get()
    }
}

pub trait BusConfig {
    const MAX_GROUPS: ValidU8;
    const MAX_MESSAGES: NonZeroU32;
    const MAX_SUBSCRIBERS: ValidU8;
}

#[macro_export]
macro_rules! define_bus_config {
    {
        $name:ident,
        max_subscribers: $subs:expr,
        max_messages: $messages:expr,
        max_groups: $groups:expr $(,)?
    } => {
        pub struct $name;

        impl $name {
            pub const MAX_MESSAGES: usize = $messages;
        }

        impl $crate::config::BusConfig for $name {
            const MAX_GROUPS: $crate::config::ValidU8 = $crate::config::ValidU8::new($groups).unwrap();
            const MAX_SUBSCRIBERS: $crate::config::ValidU8 = $crate::config::ValidU8::new($subs).unwrap();
            const MAX_MESSAGES: core::num::NonZeroU32 = core::num::NonZeroU32::new($messages).unwrap();
        }
    };
}
