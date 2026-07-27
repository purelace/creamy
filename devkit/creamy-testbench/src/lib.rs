/*
use std::num::NonZeroUsize;

use cbus::{
    BusDriver, MessageBus, SubscriberLookupData,
    core::{
        Subscriber, SubscriberId, UntypedMessage,
        buffer::{
            IncBuf, OutBuf,
            runtime::{DynIncBuf, DynOutBuf},
        },
    },
    define_bus_config,
};
use creamy_sdk::{
    api::handle_incoming,
    dispatcher::MessageHandler,
    get_incoming, get_outgoing, initialize_buffers,
    system::builtin::{
        Log, PluginAppeared, PluginDisappeared, ProtocolDeclared, ProtocolRedeclared,
        ProtocolUndeclared, StreamCancel, StreamKeepAlive,
    },
};

pub fn setup_test_harness(sub: impl Subscriber) -> Result<(), Box<dyn core::error::Error>> {
    initialize_buffers(NonZeroUsize::new(1024).unwrap())?;
    let inc = unsafe { IncBuf::from_buf(get_incoming().into_buf().as_const_sized::<1024>()) };
    let out = unsafe { OutBuf::from_buf(get_outgoing().into_buf().as_const_sized::<1024>()) };
    let mut testkit = Testkit::new();
    testkit
        .as_bus_mut()
        .add_subscriber_with(inc, out, Box::new(sub) as Box<dyn Subscriber>)?;
    Ok(())
}

define_bus_config! {
    GenericTestConfig,
    max_subscribers: 254,
    max_messages: 1024,
    max_groups: 254,
}
const M: usize = GenericTestConfig::MAX_MESSAGES;

pub struct Testkit {
    bus: MessageBus<GenericTestConfig, TestkitDriver, M>,
}

impl Default for Testkit {
    fn default() -> Self {
        Self::new()
    }
}

impl Testkit {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bus: MessageBus::new(TestkitDriver),
        }
    }

    pub const fn as_bus_mut(&mut self) -> &mut MessageBus<GenericTestConfig, TestkitDriver, M> {
        &mut self.bus
    }

    pub fn tick(&mut self, count: u8) {
        for _ in 0..count {
            self.bus.tick();
        }
    }
}

pub struct TestkitDriver;
impl BusDriver for TestkitDriver {
    fn on_subscribe(&mut self, id: SubscriberId) -> impl cbus::DataIterator {
        core::iter::once(SubscriberLookupData {
            consumer_group_id: 1,
            provider_group_id: 1,
            provider_id: id,
        })
    }

    fn on_unsubscribe(&mut self, _: SubscriberId) {}
}

pub struct TestSubscriber {
    inc: DynIncBuf,
    out: DynOutBuf,
}

impl TestSubscriber {
    pub const fn new(inc: DynIncBuf, out: DynOutBuf) -> Self {
        Self { inc, out }
    }
}

impl Subscriber for TestSubscriber {
    fn notify(&mut self) {
        unsafe {
            let dyn_buf = self.inc.into_buf();
            let inc_buf = DynIncBuf::from_buf(dyn_buf);
            handle_incoming(self, inc_buf);
        }

        //while let Some(message) = get_incoming().pop() {
        //    let mut reader = StreamReader::new(StreamId::new(1), LogReader::default());
        //}

        //let mut writer = StreamWriter::new(LogWriter::new(LogType::Info), StreamId::new(1));
        //writer.write("Йо-йо-йо 1-4-8-3 да 3-6-9, Альбукерке жжёт, чё-кого, сучара, жди сигнала");
    }
}

impl MessageHandler for TestSubscriber {
    fn handle_plugin_appeared(&mut self, message: PluginAppeared) {
        todo!()
    }

    fn handle_plugin_disappeared(&mut self, message: PluginDisappeared) {
        todo!()
    }

    fn handle_protocol_declared(&mut self, message: ProtocolDeclared) {
        todo!()
    }

    fn handle_protocol_undeclared(&mut self, message: ProtocolUndeclared) {
        todo!()
    }

    fn handle_protocol_redeclared(&mut self, message: ProtocolRedeclared) {
        todo!()
    }

    fn handle_stream_keep_alive(&mut self, message: StreamKeepAlive) {
        todo!()
    }

    fn handle_stream_cancel(&mut self, message: StreamCancel) {
        todo!()
    }

    fn handle_log(&mut self, message: Log) {
        todo!()
    }

    fn handle_unknown_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
        todo!()
    }
}
*/
