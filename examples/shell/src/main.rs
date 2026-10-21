include!(concat!(env!("OUT_DIR"), "/shell.rs"));

use std::{collections::HashMap, num::NonZeroU8};

use creamy::{
    core::{
        Constants, GroupTable,
        bus::{
            core::{
                Subscriber,
                buffer::{IncBuf, OutBuf, runtime::DynIncBuf},
            },
            define_bus_config,
        },
    },
    engine::{HostSubscriber, PluginEngine},
    sdk::{
        api::{CustomHandler, handle_incoming},
        message::UntypedMessage,
        stream::{StreamId, StreamMessage, StreamReader},
    },
};
use creamy_async_loader::{AsyncLoader, config::LoaderConfig};
use creamy_wasmtime::WasmtimeRuntime;

use self::generated::{
    dispatcher::MessageHandler,
    metadata::{PACKAGE, SPECIAL, TABLE},
    shell::setup::{ExecuteCommand, RegisterCommand},
};

pub struct ShellPlugin {
    inc: IncBuf<M>,
    out: OutBuf<M>,
    streams: HashMap<StreamId, StreamReader<RegisterCommand>>,
}

impl Subscriber for ShellPlugin {
    fn notify(&mut self) {
        unsafe {
            let dyn_buf = self.inc.as_inner_mut().as_dyn_buf();
            let inc_buf = DynIncBuf::from_buf(dyn_buf);
            handle_incoming(self, inc_buf);
        }
    }
}

impl CustomHandler for ShellPlugin {
    fn handle_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
        generated::dispatcher::dispatch_message(dispatch_value, message, self);
    }

    fn handle_on_group_enabled_event(&mut self, _: u8) {}

    fn handle_on_group_disabled_event(&mut self, _: u8) {}
}

impl MessageHandler for ShellPlugin {
    fn handle_register_command(&mut self, message: RegisterCommand) {
        let stream = self.streams.entry(message.stream_id()).or_default();
        if let Some(result) = stream.accept_message(&message) {
            match result {
                Ok(mut c) => {
                    self.streams.remove(&message.stream_id()).unwrap();
                    let bytes = c.read().read_name();
                    let command = str::from_utf8(bytes).unwrap();

                    println!("registered: {command}");
                }
                Err(err) => println!("{err}"),
            }
        }

        self.out
            .send_many_iter_exact(std::iter::once(ExecuteCommand::PREPARED.with_dst(3)));
    }

    fn handle_unknown_message(&mut self, v: u32, _: UntypedMessage) {
        println!("{v}");
    }
}

impl HostSubscriber for ShellPlugin {
    fn get_group_table(&self) -> creamy::core::GroupTable<'_> {
        GroupTable::new(&TABLE, TABLE.len() as u8, &SPECIAL, SPECIAL.len() as u8)
    }
}

pub const M: usize = 1024;
pub const S: usize = 32;
define_bus_config! {
    Legacy,
    max_subscribers: 32,
    max_messages: 1024,
    max_groups: 32,
}

struct Shell {
    engine: PluginEngine<Legacy, WasmtimeRuntime, AsyncLoader, ShellPlugin, S, M>,
}

impl Shell {
    async fn new(handle: tokio::runtime::Handle) -> Result<Self, Box<dyn core::error::Error>> {
        const HEAP_SIZE: u32 = 67_108_864;
        let mut engine = PluginEngine::new(
            Constants {
                heap_size: HEAP_SIZE,
            },
            WasmtimeRuntime::new(HEAP_SIZE)?,
            AsyncLoader::new(
                LoaderConfig {
                    parallel_downloads: 10,
                    //plugin_directory: "examples/shell/plugins".into(),
                    plugin_directory: "plugins".into(),
                }
                .into_valid()?,
                handle,
            )
            .await?,
        );

        engine.register_custom_plugin(PACKAGE, |inc, out| ShellPlugin {
            inc,
            out,
            streams: HashMap::default(),
        });
        Ok(Self { engine })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    let _ = tracing_subscriber::fmt()
        //.with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .with_target(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .try_init();

    let handle = tokio::runtime::Handle::current();
    let mut shell = Shell::new(handle).await?;
    loop {
        shell.engine.tick(NonZeroU8::new(2).unwrap());
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
    }
}
