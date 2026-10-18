use crate::{
    SubscriberId,
    api::{CustomHandler, Plugin},
    generated::{
        dispatcher::{self, MessageHandler},
        system::builtin::{GroupDeclared, GroupUndeclared, StreamCancel, StreamKeepAlive},
    },
    message::UntypedMessage,
};

static mut DST_TABLE: &mut [Option<SubscriberId>; 256] = &mut [None; 256];

#[must_use]
pub const fn get_dst_table() -> &'static [Option<SubscriberId>; 256] {
    unsafe { DST_TABLE }
}

const fn set_dst(group_id: u8, dst: u8) {
    unsafe {
        DST_TABLE[group_id as usize] = SubscriberId::new_u8(dst);
    }
}

pub struct InnerState<P: Plugin> {
    plugin: P,
}

impl<P: Plugin> InnerState<P> {
    pub const fn new(plugin: P) -> Self {
        Self { plugin }
    }

    pub fn notify(&mut self) {
        self.plugin.notify();
    }
}

impl<P: Plugin> CustomHandler for InnerState<P> {
    fn handle_message(&mut self, dispatch_value: u32, message: cbus_core::UntypedMessage) {
        dispatcher::dispatch_message(dispatch_value, message, self);
    }

    fn handle_on_group_enabled_event(&mut self, _: u8) {}

    fn handle_on_group_disabled_event(&mut self, _: u8) {}
}

impl<P: Plugin> MessageHandler for InnerState<P> {
    fn handle_group_declared(&mut self, message: GroupDeclared) {
        set_dst(message.group_id, message.provider);
        self.plugin.handle_on_group_enabled_event(message.group_id);
    }

    fn handle_group_undeclared(&mut self, message: GroupUndeclared) {
        set_dst(message.group_id, 0);
        self.plugin.handle_on_group_disabled_event(message.group_id);
    }

    fn handle_stream_keep_alive(&mut self, message: StreamKeepAlive) {}

    fn handle_stream_cancel(&mut self, message: StreamCancel) {}

    fn handle_unknown_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
        self.plugin.handle_message(dispatch_value, message);
    }
}
