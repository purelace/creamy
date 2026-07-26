#![no_std]

use core::ptr::NonNull;

use cbus::{config::BusConfig, core::Subscriber};
use creamy_devkit::BinaryPlugin;
use serde::Deserialize;

pub mod bus {
    pub use cbus::*;
}

pub mod devkit {
    pub use creamy_devkit::*;
}

pub const PACKAGE_FILE_EXTENSION: &str = "cmy";

pub trait PluginLoader {
    fn preload(&mut self);
    fn load(&mut self);
    fn loaded(&self) -> u32;
    fn take_loaded_package(&mut self) -> Option<BinaryPlugin>;
}

#[derive(Deserialize)]
pub struct Constants {
    pub heap_size: u32,
}

pub trait WasmModule: Subscriber {
    fn incoming_ptr(&self) -> NonNull<u8>;
    fn outgoing_ptr(&self) -> NonNull<u8>;
}

pub trait WasmRuntime<C: BusConfig> {
    type Error: core::error::Error;
    type Module: WasmModule;

    #[allow(clippy::missing_errors_doc)]
    fn init_module(
        &mut self,
        constants: &Constants,
        module: &[u8],
    ) -> Result<Self::Module, Self::Error>;
}
