#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![no_std]
#![allow(clippy::inline_always)]

include!(concat!(env!("OUT_DIR"), "/system.rs"));

// TODO: возможное переполнениие additional data
// сделать выравнивание данных
// проверка на общую длину
// проверка на пересечения
// сравнение длины в байтах с размером типа

extern crate alloc;
pub mod api;
pub mod message;
mod sender;
pub mod state;
pub mod stream;

pub mod binary {
    pub use creamy_binary::*;
}

pub use cbus_core::{GroupId, SubscriberId};
pub mod defines {
    pub use cbus_core::defines::*;
}

pub mod system {
    pub use crate::generated::system::*;
}

pub use sender::Sender;
