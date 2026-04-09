#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![allow(clippy::inline_always)]
#![no_std]

extern crate alloc;

pub mod buffer;
mod dst;
pub mod message;
mod subscriber;
mod untyped;

pub mod macros {
    pub use cbus_core_macros::*;
}

pub mod defines {
    /// Используем для указания длины, количества и прочего непотребства которое вряд ли будет использовать больше 32 бит во всех встроенных протоколах.
    /// Если используется больше 32 бита, то значит мы что-то делаем не так. Нормально делай, нормально будет.
    /// Можно использовать и меньшее количество байт, если протокол это позволяет, но это отностится только к пользовательским протоколам.
    pub const LENGTH_SIZE: usize = 4;

    /// Число байт, которое нам доступно для хранения данных в сообщении.
    /// Ни больше, ни меньше.
    pub const PAYLOAD_SIZE: usize = 27;

    /// Максимальный размер сообщения, который мы можем отправить или получить.
    /// Сообщение не может быть больше или меньше `MESSAGE_SIZE`.
    pub const MESSAGE_SIZE: usize = 32;
}

pub use bytemuck;
pub use dst::Destination;
pub use subscriber::{Subscriber, SubscriberId};
pub use untyped::UntypedMessage;

/// Используем для проверки в compile time что payload равен `PAYLOAD_SIZE`
#[macro_export]
macro_rules! payload_size_const_assert {
    ($payload:ty) => {
        const _ASSERT_PAYLOAD_SIZE: () = {
            assert!(
                std::mem::size_of::<$payload>() == $crate::PAYLOAD_SIZE,
                "Payload should be exactly 28 bytes!",
            );
        };
    };
}

/// Используем для проверки в compile time что размер сообщения равен `MESSAGE_SIZE`
#[macro_export]
macro_rules! message_size_const_assert {
    ($payload:ty) => {
        const _ASSERT_MESSAGE_SIZE: () = {
            assert!(
                std::mem::size_of::<$payload>() == $crate::MESSAGE_SIZE,
                "Message should be exactly 32 bytes!"
            );
        };
    };
}
