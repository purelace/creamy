#![no_std]

use cbus_core::Subscriber;
use creamy_devkit::BinaryPlugin;
use serde::Deserialize;

pub mod bus {
    pub use cbus_core::*;
}

pub mod devkit {
    pub use creamy_devkit::{BinaryPlugin, Error};
}

pub const PACKAGE_FILE_EXTENSION: &str = "cmy";

pub trait PluginLoader {
    fn preload(&mut self);
    fn loaded(&self) -> u32;
    fn take_loaded_package(&mut self) -> Option<BinaryPlugin>;
}

#[derive(Deserialize)]
pub struct Constants {
    pub heap_size: u32,
    pub buffer_size: u32,
}

pub trait WasmModule: Subscriber {
    fn incoming_ptr(&self) -> u32;
    fn outgoing_ptr(&self) -> u32;
}

pub trait WasmRuntime {
    type Error: core::error::Error;
    type Module: WasmModule;

    #[allow(clippy::missing_errors_doc)]
    fn init_module(
        &mut self,
        constants: &Constants,
        module: &[u8],
    ) -> Result<Self::Module, Self::Error>;
}
