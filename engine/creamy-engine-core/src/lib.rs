#![no_std]

extern crate alloc;

pub mod scanner;
use core::ptr::NonNull;

use cbus::core::Subscriber;

use self::scanner::ApiScanner;

pub mod bus {
    pub use cbus::*;
}

pub mod manifest {
    pub use creamy_manifest::*;
}

pub const PACKAGE_FILE_EXTENSION: &str = "cmy";

pub trait Module<const M: usize>: ApiScanner + Subscriber {
    fn incoming_ptr(&self) -> NonNull<u8>;
    fn outgoing_ptr(&self) -> NonNull<u8>;
}

#[allow(clippy::missing_errors_doc)]
pub trait Runtime<const M: usize>: 'static {
    type Error: core::error::Error;
    type Module: Module<M>;

    fn tick(&mut self);
    fn pop_ready_plugin(&mut self) -> Result<Option<Self::Module>, Self::Error>;
}
