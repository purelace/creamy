use creamy_sdk::{
    api::{CustomHandler, handle_incoming},
    bus::{
        Subscriber, UntypedMessage,
        buffer::{IncBuf, OutBuf, runtime::DynIncBuf},
    },
    dispatcher::MessageHandler,
    logging::LogReader,
    stream::{StreamId, StreamMessage, StreamReader},
    system::builtin::{
        Log, PluginAppeared, PluginDisappeared, ProtocolDeclared, ProtocolRedeclared,
        ProtocolUndeclared, StreamCancel, StreamKeepAlive,
    },
};
use rustc_hash::FxHashMap;

use crate::engine::M;

pub struct SystemPlugin {
    inc: IncBuf<M>,
    out: OutBuf<M>,
    logs: FxHashMap<StreamId, StreamReader<LogReader>>,
}

impl SystemPlugin {
    pub fn new(inc: IncBuf<M>, out: OutBuf<M>) -> Self {
        Self {
            inc,
            out,
            logs: FxHashMap::default(),
        }
    }
}

impl Subscriber for SystemPlugin {
    fn notify(&mut self) {
        unsafe {
            let dyn_buf = self.inc.as_inner_mut().as_dyn_buf();
            let inc_buf = DynIncBuf::from_buf(dyn_buf);
            handle_incoming(self, inc_buf);
        }
    }
}

impl CustomHandler for SystemPlugin {
    fn handle_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
        creamy_sdk::dispatcher::dispatch_message(dispatch_value, message, self);
    }
}

impl MessageHandler for SystemPlugin {
    // Send, not receive
    fn handle_plugin_appeared(&mut self, _: PluginAppeared) {}
    fn handle_plugin_disappeared(&mut self, _: PluginDisappeared) {}
    fn handle_protocol_declared(&mut self, _: ProtocolDeclared) {}
    fn handle_protocol_undeclared(&mut self, _: ProtocolUndeclared) {}
    fn handle_protocol_redeclared(&mut self, _: ProtocolRedeclared) {}
    fn handle_stream_keep_alive(&mut self, _: StreamKeepAlive) {}
    fn handle_stream_cancel(&mut self, _: StreamCancel) {}

    fn handle_log(&mut self, message: Log) {
        let stream = self
            .logs
            .entry(message.stream_id())
            .or_insert(StreamReader::new(message.stream_id(), LogReader::default()));

        let result = match stream.read(message) {
            Ok(value) => value,
            Err(e) => panic!("{e}"),
        };

        let stream = self.logs.remove(&message.stream_id()).unwrap();

        if result {
            let message = stream.into_reader().into_string().unwrap();
            tracing::info!("log: {message}");
        }
    }

    fn handle_unknown_message(&mut self, dispatch_value: u32, message: UntypedMessage) {}
}
