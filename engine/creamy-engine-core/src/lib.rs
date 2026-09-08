#![no_std]

use core::{num::NonZeroU8, ptr::NonNull};

use cbus::{config::BusConfig, core::Subscriber};
use creamy_devkit::BinaryPlugin;
use creamy_phf::get_hash;
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

pub struct GroupTable<'a> {
    table: &'a [u8],
    table_size: u8,
    special: &'a [u64],
    special_size: u8,
}

impl<'a> GroupTable<'a> {
    #[must_use]
    pub const fn new(
        table: &'a [u8],
        table_size: u8,
        special: &'a [u64],
        special_size: u8,
    ) -> Self {
        Self {
            table,
            table_size,
            special,
            special_size,
        }
    }

    pub fn get_group_id(&self, path: &str) -> NonZeroU8 {
        let group = get_hash(0xdead_beef, path.as_bytes()) % u64::from(self.special_size);
        let perfect_seed = self.special[group as usize];
        let hash = get_hash(perfect_seed, path.as_bytes());
        let id = self.table[(hash % u64::from(self.table_size)) as usize];
        NonZeroU8::new(id).unwrap()
    }
}

pub trait WasmModule: Subscriber {
    fn incoming_ptr(&self) -> NonNull<u8>;
    fn outgoing_ptr(&self) -> NonNull<u8>;
    fn get_group_table(&mut self) -> GroupTable<'_>;
}

pub trait WasmRuntime<C: BusConfig>: 'static {
    type Error: core::error::Error;
    type Module: WasmModule;

    #[allow(clippy::missing_errors_doc)]
    fn init_module(
        &mut self,
        constants: &Constants,
        module: &[u8],
    ) -> Result<Self::Module, Self::Error>;
}
