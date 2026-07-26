#![no_std]

extern crate alloc;

use core::ptr::NonNull;

use creamy_engine_core::{
    Constants, WasmModule, WasmRuntime,
    bus::{config::BusConfig, core::Subscriber},
};
use wasmtime::{Config, Engine, Instance, Module, Store, TypedFunc};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("{0}")]
    Wasmtime(#[from] wasmtime::Error),
}

pub struct WasmPlugin {
    incoming_ptr: NonNull<u8>,
    outgoing_ptr: NonNull<u8>,
    notify: TypedFunc<(), ()>,
    store: Store<()>,
    instance: Instance,
}

impl WasmModule for WasmPlugin {
    fn incoming_ptr(&self) -> NonNull<u8> {
        self.incoming_ptr
    }

    fn outgoing_ptr(&self) -> NonNull<u8> {
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

impl<C: BusConfig> WasmRuntime<C> for WasmtimeRuntime {
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

        let export_incoming_buffer =
            instance.get_typed_func::<u32, u32>(&mut store, "internal__export_incoming_buffer")?;
        let incoming_ptr = export_incoming_buffer.call(&mut store, C::MAX_MESSAGES.get())?;

        let export_outgoing_buffer =
            instance.get_typed_func::<u32, u32>(&mut store, "internal__export_outgoing_buffer")?;
        let outgoing_ptr = export_outgoing_buffer.call(&mut store, C::MAX_MESSAGES.get())?;

        let init_func = instance.get_typed_func::<u32, u32>(&mut store, "internal__init_plugin")?;
        let result = init_func.call(&mut store, constants.heap_size)?;
        assert!(result == 0, "plugin init error");

        let notify_func = instance.get_typed_func::<(), ()>(&mut store, "internal__notify")?;

        tracing::debug!("iptr: {incoming_ptr}, optr: {outgoing_ptr}");

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(wasmtime::format_err!("failed to find `memory` export"))?;
        let start = memory.data_ptr(&mut store) as usize;
        let host_inc_ptr = (start + incoming_ptr as usize) as *mut u8;
        let host_out_ptr = (start + outgoing_ptr as usize) as *mut u8;

        Ok(WasmPlugin {
            incoming_ptr: NonNull::new(host_inc_ptr)
                .ok_or(wasmtime::format_err!("Pointer of incoming buffer is null"))?,
            outgoing_ptr: NonNull::new(host_out_ptr)
                .ok_or(wasmtime::format_err!("Pointer of outgoing buffer is null"))?,
            notify: notify_func,
            store,
            instance,
        })
    }
}
