#![no_std]

extern crate alloc;

mod table;
use alloc::vec::Vec;
use core::ptr::NonNull;

use cbus::core::Subscriber;
use creamy_plugin_package::{
    manifest::Manifest,
    model::{Version, definition::ProtocolModel},
    pool::StringPool,
};
pub use table::GroupTable;

pub mod bus {
    pub use cbus::*;
}

pub mod package {
    pub use creamy_plugin_package::*;
}

pub const PACKAGE_FILE_EXTENSION: &str = "cmy";

pub struct Metadata {
    pub version: Version,
    pub manifest: Manifest,
    pub pool: StringPool,
    pub models: Vec<ProtocolModel>,
}

pub trait Module<const M: usize>: Subscriber {
    fn incoming_ptr(&self) -> NonNull<u8>;
    fn outgoing_ptr(&self) -> NonNull<u8>;
    fn get_group_table(&mut self) -> GroupTable<'_>;
}

#[allow(clippy::missing_errors_doc)]
pub trait Runtime<const M: usize>: 'static {
    type Error: core::error::Error;
    type Module: Module<M>;

    fn tick(&mut self);
    fn pop_ready_plugin(&mut self) -> Result<Option<(Metadata, Self::Module)>, Self::Error>;
}
