#![no_std]

include!(concat!(env!("OUT_DIR"), "/ping.rs"));

use creamy_sdk::{Sender, api::Plugin, declare_plugin, error, info, warn};

use self::generated::dispatcher::{GroupLifecycleHook, MessageHandler};
use crate::generated::ping::messages::{Ping, Pong};

declare_plugin!(PingPlugin, generated);

struct PingPlugin {
    sender: Sender,
}

impl Plugin for PingPlugin {
    fn init(sender: Sender) -> Option<Self> {
        info!("Hello, World!");
        warn!("Дарова, заебал!");
        error!("Как дела?");
        Some(Self { sender })
    }

    fn notify(&mut self) {
        info!("Йо-йо-йо 1-4-8-3 да 3-6-9, Альбукерке жжёт, чё-кого, сучара, жди сигнала");
    }
}

impl GroupLifecycleHook for PingPlugin {
    fn on_messages_group_enabled(&mut self) {}

    fn on_messages_group_disabled(&mut self) {}
}

impl MessageHandler for PingPlugin {
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

    fn handle_unknown_message(
        &mut self,
        _dispatch_value: u32,
        _message: creamy_sdk::message::UntypedMessage,
    ) {
    }
}
