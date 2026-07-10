use cbus_core::UntypedMessage;

use crate::{
    api::{CustomHandler, Plugin},
    dispatcher::{self, MessageHandler},
    system::builtin::{
        Log, PluginAppeared, PluginDisappeared, ProtocolDeclared, ProtocolRedeclared,
        ProtocolUndeclared, StreamCancel, StreamKeepAlive,
    },
};

pub struct InnerState<P: Plugin> {
    plugin: P,
}

impl<P: Plugin> InnerState<P> {
    pub const fn new(plugin: P) -> Self {
        Self { plugin }
    }
}

impl<P: Plugin> CustomHandler for InnerState<P> {
    #[inline(always)]
    fn handle_message(&mut self, dispatch_value: u32, message: cbus_core::UntypedMessage) {
        dispatcher::dispatch_message(dispatch_value, message, self);
    }
}

impl<P: Plugin> MessageHandler for InnerState<P> {
    #[inline(always)]
    fn handle_plugin_appeared(&mut self, message: PluginAppeared) {}

    #[inline(always)]
    fn handle_plugin_disappeared(&mut self, message: PluginDisappeared) {}

    #[inline(always)]
    fn handle_protocol_declared(&mut self, message: ProtocolDeclared) {
        let x: Option<u8>;
        let y: u8;
    }

    #[inline(always)]
    fn handle_protocol_undeclared(&mut self, message: ProtocolUndeclared) {}

    #[inline(always)]
    fn handle_protocol_redeclared(&mut self, message: ProtocolRedeclared) {}

    #[inline(always)]
    fn handle_stream_keep_alive(&mut self, message: StreamKeepAlive) {}

    #[inline(always)]
    fn handle_stream_cancel(&mut self, message: StreamCancel) {}

    fn handle_log(&mut self, _: Log) {}

    #[inline(always)]
    fn handle_unknown_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
        self.plugin.handle_message(dispatch_value, message);
    }
}
