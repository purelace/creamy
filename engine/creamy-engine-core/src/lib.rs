#![no_std]

use cbus::{
    config::{BusConfig, ValidConfig},
    core::Subscriber,
};
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
    fn loaded(&self) -> u32;
    fn take_loaded_package(&mut self) -> Option<BinaryPlugin>;
}

#[derive(Deserialize)]
pub struct Constants {
    pub heap_size: u32,
    pub buffer_size: u32,

    pub max_messages: u32,
    pub max_groups: u8,
    pub max_subscribers: u8,
}

impl BusConfig for Constants {
    fn max_subscribers(&self) -> u8 {
        self.max_subscribers
    }

    fn max_messages(&self) -> usize {
        //TODO: remove usize cast operation
        self.max_messages as usize
    }

    fn max_groups(&self) -> u8 {
        self.max_groups
    }

    fn into_valid(self) -> Result<cbus::config::ValidConfig<Self>, cbus::BusError> {
        ValidConfig::new(self)
    }
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
