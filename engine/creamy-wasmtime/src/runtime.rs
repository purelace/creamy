use core::ptr::NonNull;

use creamy_engine_core::{Metadata, Runtime};
use wasmtime::{Config, Engine, Instance, PoolingAllocationConfig, Store};

use crate::{
    error::RuntimeError,
    loader::{Loader, PluginLoader},
    plugin::WasmPlugin,
};

pub struct WTRuntime {
    heap_size: u32,
    engine: Engine,
    loader: Loader,
}

impl WTRuntime {
    fn new_config(heap_size: u32) -> Config {
        let mut config = Config::default();
        let mut pooling = PoolingAllocationConfig::new();
        pooling.max_memory_size(heap_size as usize);
        config.compiler_inlining(wasmtime::Inlining::Yes);
        config.allocation_strategy(pooling);
        config
    }

    /// Creates a new [`WasmtimeRuntime`].
    ///
    /// # Errors
    ///
    /// See [`wasmtime::engine::Engine::new()`].
    pub fn new(heap_size: u32, loader: Loader) -> Result<Self, RuntimeError> {
        Ok(Self {
            heap_size,
            engine: Engine::new(&Self::new_config(heap_size))?,
            loader,
        })
    }

    #[must_use]
    pub const fn loader(&self) -> &Loader {
        &self.loader
    }

    pub const fn loader_mut(&mut self) -> &mut Loader {
        &mut self.loader
    }
}

impl<const M: usize> Runtime<M> for WTRuntime {
    type Error = RuntimeError;
    type Module = WasmPlugin;

    fn tick(&mut self) {
        self.loader.tick();
    }

    fn pop_ready_plugin(&mut self) -> Result<Option<(Metadata, Self::Module)>, Self::Error> {
        let Some(package) = self.loader.take_loaded_package() else {
            return Ok(None);
        };

        let mut store = Store::new(&self.engine, ());
        let module = wasmtime::Module::from_binary(&self.engine, package.core())?;
        let instance = Instance::new(&mut store, &module, &[])?;

        let export_incoming_buffer =
            instance.get_typed_func::<u32, u32>(&mut store, "internal__export_incoming_buffer")?;
        let incoming_ptr = export_incoming_buffer.call(&mut store, M as u32)?;

        let export_outgoing_buffer =
            instance.get_typed_func::<u32, u32>(&mut store, "internal__export_outgoing_buffer")?;
        let outgoing_ptr = export_outgoing_buffer.call(&mut store, M as u32)?;

        let init_func = instance.get_typed_func::<u32, u32>(&mut store, "internal__init_plugin")?;
        let result = init_func.call(&mut store, self.heap_size)?;
        assert!(result == 0, "plugin init error");

        let notify_func = instance.get_typed_func::<(), ()>(&mut store, "internal__notify")?;

        tracing::debug!("iptr: {incoming_ptr}, optr: {outgoing_ptr}");

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(wasmtime::format_err!("failed to find `memory` export"))?;
        let start = memory.data_ptr(&mut store) as usize;
        let host_inc_ptr = (start + incoming_ptr as usize) as *mut u8;
        let host_out_ptr = (start + outgoing_ptr as usize) as *mut u8;

        Ok(Some((
            Metadata {
                version: package.version,
                manifest: package.manifest,
                pool: package.pool,
                models: package.models,
            },
            WasmPlugin {
                incoming_ptr: NonNull::new(host_inc_ptr)
                    .ok_or(wasmtime::format_err!("Pointer of incoming buffer is null"))?,
                outgoing_ptr: NonNull::new(host_out_ptr)
                    .ok_or(wasmtime::format_err!("Pointer of outgoing buffer is null"))?,
                notify: notify_func,
                store,
                instance,
            },
        )))
    }
}
