#![no_std]

use creamy_sdk::{Sender, api::Plugin, declare_plugin, message::UntypedMessage};

use self::generated::dispatcher::{GroupLifecycleHook, MessageHandler};

include!(concat!(env!("OUT_DIR"), "/empty.rs"));

declare_plugin!(EmptyPlugin, generated);

struct EmptyPlugin;

impl Plugin for EmptyPlugin {
    fn init(_: Sender) -> Option<Self> {
        Some(Self)
    }
}

impl GroupLifecycleHook for EmptyPlugin {
    fn on_none_group_enabled(&mut self) {}

    fn on_none_group_disabled(&mut self) {}
}

impl MessageHandler for EmptyPlugin {
    fn handle_unknown_message(&mut self, _: u32, _: UntypedMessage) {}
}
