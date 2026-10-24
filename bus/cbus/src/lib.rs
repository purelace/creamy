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
pub mod config;
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

/*
 TODO: Проблема диапазона
 TODO: Обрабатывать возможное переполнение
*/
