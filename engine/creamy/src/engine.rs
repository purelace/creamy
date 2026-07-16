use std::time::Duration;

use cbus::{MessageBus, config::Legacy};
use creamy_cbus_driver::CreamyDriver;
use creamy_engine_core::{Constants, PluginLoader, WasmRuntime};

pub struct PluginEngine<R: WasmRuntime, L: PluginLoader> {
    bus: MessageBus<CreamyDriver, R::Module>,
    constants: Constants,
    runtime: R,
    loader: L,
}

//#[allow(dead_code, unused)]
impl<R: WasmRuntime, L: PluginLoader> PluginEngine<R, L> {
    pub fn new(
        constants: Constants,
        runtime: R,
        loader: L,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            constants,
            runtime,
            loader,
            bus: MessageBus::new(Legacy, CreamyDriver::new)?,
        })
    }

    pub fn run(&mut self) {
        while self.loader.loaded() != 0
            && let Some(package) = self.loader.take_loaded_package()
        {
            let module = self
                .runtime
                .init_module(&self.constants, package.core())
                .unwrap();
            self.bus.add_subscriber(|_, _| module).unwrap();
        }

        self.bus.tick();
        self.bus.tick();

        // For testing only
        std::thread::sleep(Duration::from_millis(16));
    }

    #[must_use]
    pub const fn loaded_plugins(&self) -> u8 {
        self.bus.subscribers()
    }
}
