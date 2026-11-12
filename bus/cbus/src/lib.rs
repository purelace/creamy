#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![no_std]
#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::todo)]
#![deny(clippy::pedantic)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::unreachable)]
#![allow(clippy::inline_always)]
#![allow(clippy::cast_ptr_alignment)]

extern crate alloc;

mod bus;
mod cpu;
mod driver;
mod error;
mod lookup;
mod sys;

pub mod core {
    pub use cbus_core::*;
}

pub use bus::MessageBus;
pub use driver::{BusDriver, DataIterator};
pub use error::BusError;
pub use lookup::SubscriberLookupData;

#[macro_export]
macro_rules! define_bus {
    (
        max_messages: $mm: expr,
        max_protocols: $mg: expr,
        max_subscribers: $ms: expr,
    ) => {
        extern crate alloc;
        pub const MAX_MESSAGES: usize = $mm;
        pub const MAX_PROTOCOLS: usize = $mg;
        pub const MAX_SUBSCRIBERS: usize = $ms;
        pub type IncBuf = $crate::core::buffer::IncBuf<$mm>;
        pub type OutBuf = $crate::core::buffer::OutBuf<$mm>;
        pub type MessageBus<D, S = alloc::boxed::Box<dyn $crate::core::Subscriber>> =
            $crate::MessageBus<D, $mm, $mg, $ms, S>;
    };
}

/*
 TODO: Проблема диапазона
 TODO: Обрабатывать возможное переполнение
*/
