use std::ops::Deref;

use cbus_core::defines::MESSAGE_SIZE;

use crate::{BusError, defines::METADATA};

pub trait BusConfig: Sized {
    fn max_subscribers(&self) -> u8;
    fn max_messages(&self) -> usize;
    fn max_groups(&self) -> u8;

    /// Validates and wraps the provided configuration into a bus-ready instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid:
    /// * [`BusError::ValueTooSmall`] — if `max_messages` is less than 1024.
    /// * [`BusError::ValueOutOfRange`] — if any parameter exceeds architectural limits.
    /// * [`BusError::GroupsIsNotMultipleOf2`] - if `max_groups` is not a multiple of 2
    fn into_valid(self) -> Result<ValidConfig<Self>, BusError>;
}

pub struct ValidConfig<C: BusConfig>(C);
impl<C: BusConfig> Deref for ValidConfig<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: BusConfig> ValidConfig<C> {
    /// Validates and wraps the provided configuration into a bus-ready instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid:
    /// * [`BusError::ValueTooSmall`] — if `max_messages` is less than 1024.
    /// * [`BusError::ValueOutOfRange`] — if any parameter exceeds architectural limits.
    /// * [`BusError::GroupsIsNotMultipleOf2`] - if `max_groups` is not a multiple of 2
    pub fn new(config: C) -> Result<Self, BusError> {
        if config.max_subscribers() == 0 {
            return Err(BusError::ValueTooSmall {
                name: "max_subscribers".to_string(),
                current: 0,
                min: 2,
            });
        }

        if !config.max_subscribers().is_multiple_of(2) {
            return Err(BusError::ValueIsNotMultipleOf2 {
                name: "max_subscribers".to_string(),
            });
        }

        if config.max_messages() < 1024 {
            return Err(BusError::ValueTooSmall {
                name: "max_messages".to_string(),
                current: config.max_messages(),
                min: 1024,
            });
        }

        if !config.max_messages().is_multiple_of(2) {
            return Err(BusError::ValueIsNotMultipleOf2 {
                name: "max_messages".to_string(),
            });
        }

        let Some(slice_size) = config.max_messages().checked_mul(MESSAGE_SIZE) else {
            return Err(BusError::ValueTooBig {
                name: "max_messages".to_string(),
            });
        };

        let Some(slice_size) = slice_size.checked_add(METADATA) else {
            return Err(BusError::ValueTooBig {
                name: "max_messages".to_string(),
            });
        };

        let Some(total_size) = slice_size.checked_mul(config.max_subscribers() as usize) else {
            return Err(BusError::ValueTooBig {
                name: "max_messages".to_string(),
            });
        };

        const MAX_SIZE: usize = usize::MAX / 2 + 1;
        if total_size > MAX_SIZE {
            return Err(BusError::TooBigPoolSize {
                current: total_size,
                max: MAX_SIZE,
            });
        }

        if !config.max_groups().is_power_of_two() {
            return Err(BusError::GroupsIsNotPowerOf2);
        }

        Ok(Self(config))
    }
}

#[macro_export]
macro_rules! define_bus_config {
    {
        $name:ident,
        max_subscribers: $subs:expr,
        max_messages: $msg:expr,
        max_groups: $grps:expr $(,)?
    } => {
        pub struct $name;
        impl BusConfig for $name {
            fn max_subscribers(&self) -> u8 {
                $subs
            }

            fn max_messages(&self) -> usize {
                $msg
            }

            fn max_groups(&self) -> u8 {
                $grps
            }

            fn into_valid(self) -> Result<ValidConfig<Self>, BusError> {
                ValidConfig::new(self)
            }
        }
    };
}

define_bus_config! {
    Legacy,
    max_subscribers: 32,
    max_messages: 1024,
    max_groups: 64
}

define_bus_config! {
    Middle,
    max_subscribers: 128,
    max_messages: 2048,
    max_groups: 128
}

define_bus_config! {
    Advanced,
    max_subscribers: u8::MAX - 1,
    max_messages: 2048,
    max_groups: 128,
}
