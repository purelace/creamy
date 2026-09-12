#![no_std]

use creamy_sdk::{Sender, api::Plugin, declare_plugin, message::UntypedMessage};

use self::generated::{
    dispatcher::{GroupLifecycleHook, MessageHandler},
    ping::messages::{Ping, Pong},
};

include!(concat!(env!("OUT_DIR"), "/pong.rs"));
declare_plugin!(PongPlugin, generated);

struct PongPlugin {
    sender: Sender,
}

impl Plugin for PongPlugin {
    fn init(sender: Sender) -> Option<Self> {
        Some(Self { sender })
    }
}

impl GroupLifecycleHook for PongPlugin {
    fn on_messages_group_enabled(&mut self) {}

    fn on_messages_group_disabled(&mut self) {}
}

impl MessageHandler for PongPlugin {
    fn handle_ping(&mut self, message: Ping) {
        assert!(
            self.sender.send_to(
                &Pong::PREPARED
                    .with_dst(message.src)
                    .with_serial(message.serial),
            )
        );
    }

    fn handle_pong(&mut self, message: Pong) {
        assert!(
            self.sender.send_to(
                &Ping::PREPARED
                    .with_dst(message.src)
                    .with_serial(message.serial),
            )
        );
    }

    fn handle_unknown_message(&mut self, _: u32, _: UntypedMessage) {}
}
