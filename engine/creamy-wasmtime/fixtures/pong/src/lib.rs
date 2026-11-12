#![no_std]

use creamy_sdk::{Sender, api::Plugin};
use creamy_sdk_wasm::declare_plugin;

use self::generated::{
    dispatcher::{Dispatcher, MessageHandler},
    ping::{Ping, Pong},
};

include!(concat!(env!("OUT_DIR"), "/pong.rs"));
declare_plugin!(PongPlugin);

struct PongPlugin {
    sender: Sender,
}

impl Plugin for PongPlugin {
    type CustomData = ();
    type Delegate = Dispatcher<Self>;
    fn init(sender: Sender, _: Self::CustomData) -> Option<Self> {
        Some(Self { sender })
    }
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
}
