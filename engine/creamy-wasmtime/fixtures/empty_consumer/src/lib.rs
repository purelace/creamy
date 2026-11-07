#![no_std]

use creamy_sdk::{Sender, api::Plugin, message::UntypedMessage};
use creamy_sdk_wasm::declare_plugin;

use self::generated::dispatcher::{GroupLifecycleHook, MessageHandler};

include!(concat!(env!("OUT_DIR"), "/empty_consumer.rs"));

declare_plugin!(EmptyPlugin, generated);

struct EmptyPlugin;

impl Plugin for EmptyPlugin {
    type CustomData = ();
    fn init(_: Sender, _: Self::CustomData) -> Option<Self> {
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
