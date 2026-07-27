#![no_std]

extern crate alloc;

use alloc::boxed::Box;

use creamy_sdk::{
    api::{CustomHandler, handle_incoming},
    bus::{
        Subscriber, SubscriberId, UntypedMessage,
        buffer::{IncBuf, OutBuf, runtime::DynIncBuf},
    },
    dispatcher::MessageHandler,
    logging::LogReader,
    stream::{StreamId, StreamMessage, StreamReader},
    system::builtin::{
        Log, LogType, PluginAppeared, PluginDisappeared, ProtocolDeclared, ProtocolRedeclared,
        ProtocolUndeclared, StreamCancel, StreamKeepAlive,
    },
};
use rustc_hash::FxHashMap;

pub struct SystemPlugin<const M: usize, S: CustomHandler> {
    inc: IncBuf<M>,
    out: OutBuf<M>,
    logs: FxHashMap<StreamId, StreamReader<LogReader>>,

    names: FxHashMap<SubscriberId, Box<str>>,

    extend: S,
}

impl<const M: usize, S: CustomHandler> SystemPlugin<M, S> {
    #[must_use]
    pub fn new(inc: IncBuf<M>, out: OutBuf<M>, extend: S) -> Self {
        Self {
            inc,
            out,
            logs: FxHashMap::default(),
            names: FxHashMap::default(),
            extend,
        }
    }

    pub fn add_plugin_name(&mut self, id: SubscriberId, name: impl Into<Box<str>>) {
        self.names.insert(id, name.into());
    }

    pub fn remove_plugin_name(&mut self, id: SubscriberId) {
        self.names.remove(&id);
    }
}

impl<const M: usize, H: CustomHandler> Subscriber for SystemPlugin<M, H> {
    fn notify(&mut self) {
        unsafe {
            let dyn_buf = self.inc.as_inner_mut().as_dyn_buf();
            let inc_buf = DynIncBuf::from_buf(dyn_buf);
            handle_incoming(self, inc_buf);
        }
    }
}

impl<const M: usize, H: CustomHandler> CustomHandler for SystemPlugin<M, H> {
    fn handle_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
        creamy_sdk::dispatcher::dispatch_message(dispatch_value, message, self);
    }
}

impl<const M: usize, H: CustomHandler> MessageHandler for SystemPlugin<M, H> {
    // Send, not receive
    fn handle_plugin_appeared(&mut self, _: PluginAppeared) {}
    fn handle_plugin_disappeared(&mut self, _: PluginDisappeared) {}
    fn handle_protocol_declared(&mut self, _: ProtocolDeclared) {}
    fn handle_protocol_undeclared(&mut self, _: ProtocolUndeclared) {}
    fn handle_protocol_redeclared(&mut self, _: ProtocolRedeclared) {}
    fn handle_stream_keep_alive(&mut self, _: StreamKeepAlive) {}
    fn handle_stream_cancel(&mut self, _: StreamCancel) {}

    // Receive, not send
    fn handle_log(&mut self, message: Log) {
        let stream = self
            .logs
            .entry(message.stream_id())
            .or_insert(StreamReader::new(message.stream_id(), LogReader::default()));

        let result = match stream.read(message) {
            Ok(value) => value,
            Err(e) => panic!("{e}"),
        };

        if !result {
            return;
        }

        let stream = self.logs.remove(&message.stream_id()).unwrap();
        let reader = stream.into_reader();
        let log_type = reader.log_type();
        let content = reader.into_string().unwrap();

        let target = self
            .names
            .get(&unsafe { SubscriberId::new_unchecked(message.src) })
            .unwrap();

        match log_type {
            LogType::Debug => log::debug!(target: target, "{content}"),
            LogType::Info => log::info!(target: target, "{content}"),
            LogType::Warning => log::warn!(target:target, "{content}"),
            LogType::Error => log::error!(target: target, "{content}"),
        }
    }

    fn handle_unknown_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
        self.extend.handle_message(dispatch_value, message);
    }
}
