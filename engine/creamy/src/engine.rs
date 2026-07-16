use std::{num::NonZeroU8, time::Duration};

use cbus::{MessageBus, config::ValidConfig};
use creamy_cbus_driver::CreamyDriver;
use creamy_engine_core::{Constants, PluginLoader, WasmRuntime};

pub struct PluginEngine<R: WasmRuntime, L: PluginLoader> {
    bus: MessageBus<CreamyDriver, R::Module>,
    constants: ValidConfig<Constants>,
    runtime: R,
    loader: L,
}

impl<R: WasmRuntime, L: PluginLoader> PluginEngine<R, L> {
    pub fn new(constants: ValidConfig<Constants>, runtime: R, loader: L) -> Self {
        Self {
            bus: MessageBus::new(&constants, CreamyDriver::new),
            constants,
            runtime,
            loader,
        }
    }

    pub fn run(&mut self, roundtrip: NonZeroU8) {
        while self.loader.loaded() != 0
            && let Some(package) = self.loader.take_loaded_package()
        {
            let module = self
                .runtime
                .init_module(&self.constants, package.core())
                .unwrap();
            self.bus.add_subscriber(|_, _| module).unwrap();
        }

        for _ in 0..roundtrip.get() {
            self.bus.tick();
        }

        // For testing only
        std::thread::sleep(Duration::from_millis(16));
    }

    #[must_use]
    pub const fn loaded_plugins(&self) -> u8 {
        self.bus.subscribers()
    }
}
