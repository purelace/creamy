#![no_std]

extern crate alloc;

use creamy_engine_core::{Constants, WasmModule, WasmRuntime, bus::core::Subscriber};
use wasmtime::{Config, Engine, Instance, Module, Store, TypedFunc};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("{0}")]
    Wasmtime(#[from] wasmtime::Error),
}

pub struct WasmPlugin {
    incoming_ptr: u32,
    outgoing_ptr: u32,
    notify: TypedFunc<(), ()>,
    store: Store<()>,
    instance: Instance,
}

impl WasmModule for WasmPlugin {
    fn incoming_ptr(&self) -> u32 {
        self.incoming_ptr
    }

    fn outgoing_ptr(&self) -> u32 {
        self.outgoing_ptr
    }
}

impl Subscriber for WasmPlugin {
    fn notify(&mut self) {
        self.notify.call(&mut self.store, ()).unwrap();
    }
}

pub struct WasmtimeRuntime {
    engine: Engine,
}

impl WasmtimeRuntime {
    pub fn new() -> Result<Self, RuntimeError> {
        let mut config = Config::default();
        config.compiler_inlining(wasmtime::Inlining::Yes);
        Ok(Self {
            engine: Engine::new(&config)?,
        })
    }
}

impl WasmRuntime for WasmtimeRuntime {
    type Error = RuntimeError;
    type Module = WasmPlugin;

    fn init_module(
        &mut self,
        constants: &Constants,
        module: &[u8],
    ) -> Result<Self::Module, Self::Error> {
        let module = Module::from_binary(&self.engine, module)?;
        let mut store = Store::new(&self.engine, ());

        let instance = Instance::new(&mut store, &module, &[])?;

        let init_func = instance.get_typed_func::<u32, u32>(&mut store, "internal__init_plugin")?;
        let result = init_func.call(&mut store, constants.heap_size)?;
        assert!(result == 0, "plugin init error");

        let export_incoming_buffer =
            instance.get_typed_func::<u32, u32>(&mut store, "internal__export_incoming_buffer")?;
        let incoming_ptr = export_incoming_buffer.call(&mut store, constants.buffer_size)?;

        let export_outgoing_buffer =
            instance.get_typed_func::<u32, u32>(&mut store, "internal__export_outgoing_buffer")?;
        let outgoing_ptr = export_outgoing_buffer.call(&mut store, constants.buffer_size)?;

        let notify_func = instance.get_typed_func::<(), ()>(&mut store, "internal__notify")?;

        tracing::debug!("iptr: {incoming_ptr}, optr: {outgoing_ptr}");

        Ok(WasmPlugin {
            incoming_ptr,
            outgoing_ptr,
            notify: notify_func,
            store,
            instance,
        })
    }
}
