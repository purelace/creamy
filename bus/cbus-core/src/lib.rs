#![no_std]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::unreachable)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(unused)]

extern crate alloc;

pub mod buffer;
pub mod message;
mod subscriber;
mod untyped;

pub mod defines {
    /// Используем для указания длины, количества и прочего непотребства которое вряд ли будет использовать больше 32 бит во всех встроенных протоколах.
    /// Если используется больше 32 бита, то значит мы что-то делаем не так. Нормально делай, нормально будет.
    /// Можно использовать и меньшее количество байт, если протокол это позволяет, но это отностится только к пользовательским протоколам.
    pub const LENGTH_SIZE: usize = 4;

    /// Число байт, которое нам доступно для хранения данных в сообщении.
    /// Ни больше, ни меньше.
    pub const PAYLOAD_SIZE: usize = 28;

    /// Максимальный размер сообщения, который мы можем отправить или получить.
    /// Сообщение не может быть больше или меньше `MESSAGE_SIZE`.
    pub const MESSAGE_SIZE: usize = 32;

    pub const METADATA: usize = 64;
    pub const TARGET_ALIGN: usize = 64;
}

pub use subscriber::{Subscriber, SubscriberId};
pub use untyped::UntypedMessage;
