use cbus::{
    MessageBus,
    config::{BusConfig, Legacy},
    core::Subscriber,
};
use creamy_cbus_driver::CreamyDriver;
use wasmtime::{Config, Engine, Instance, Module, Store, TypedFunc};

pub enum EngineSubscribers<S: Subscriber> {
    Builtin(S),
    Wasm(WasmPlugin),
}

impl<S: Subscriber> Subscriber for EngineSubscribers<S> {
    fn notify(&mut self) {
        match self {
            EngineSubscribers::Builtin(p) => p.notify(),
            EngineSubscribers::Wasm(p) => p.notify(),
        }
    }
}

impl<S: Subscriber> From<WasmPlugin> for EngineSubscribers<S> {
    fn from(value: WasmPlugin) -> Self {
        EngineSubscribers::Wasm(value)
    }
}

pub struct WasmPlugin {
    incoming_ptr: u32,
    outgoing_ptr: u32,
    notify: TypedFunc<(), ()>,
    store: Store<ModuleState>,
    instance: Instance,
}

impl Subscriber for WasmPlugin {
    fn notify(&mut self) {
        self.notify.call(&mut self.store, ()).unwrap();
    }
}

pub struct ModuleState {}

pub struct Runtime<S: Subscriber> {
    engine: Engine,
    bus: MessageBus<CreamyDriver, EngineSubscribers<S>>,

    /* Read-only fields */
    heap_size: u32,
    messages: u32,
}

impl<S: Subscriber> Runtime<S> {
    pub fn new(heap_size: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Config::default();
        config.compiler_inlining(wasmtime::Inlining::Yes);
        Ok(Self {
            engine: Engine::new(&config)?,
            bus: MessageBus::new(Legacy, CreamyDriver::new)?,
            heap_size,
            messages: Legacy.max_messages() as u32,
        })
    }

    pub fn init_module(&mut self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let module = Module::from_binary(&self.engine, bytes)?;
        let mut store = Store::new(&self.engine, ModuleState {});

        let instance = Instance::new(&mut store, &module, &[])?;

        let init_func = instance.get_typed_func::<u32, u32>(&mut store, "internal__init_plugin")?;
        let result = init_func.call(&mut store, self.heap_size)?;
        assert!(result == 0, "plugin init error");

        let export_incoming_buffer =
            instance.get_typed_func::<u32, u32>(&mut store, "internal__export_incoming_buffer")?;
        let incoming_ptr = export_incoming_buffer.call(&mut store, self.messages)?;

        let export_outgoing_buffer =
            instance.get_typed_func::<u32, u32>(&mut store, "internal__export_outgoing_buffer")?;
        let outgoing_ptr = export_outgoing_buffer.call(&mut store, self.messages)?;

        let notify_func = instance.get_typed_func::<(), ()>(&mut store, "internal__notify")?;

        println!("iptr: {incoming_ptr}, optr: {outgoing_ptr}");

        self.bus.add_subscriber(|_, _| WasmPlugin {
            incoming_ptr,
            outgoing_ptr,
            notify: notify_func,
            store,
            instance,
        })?;

        Ok(())
    }

    pub fn tick(&mut self) {
        self.bus.tick();
    }

    pub const fn loaded_plugins(&self) -> u8 {
        self.bus.subscribers()
    }
}
