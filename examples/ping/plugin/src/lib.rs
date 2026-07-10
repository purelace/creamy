#![allow(clippy::inline_always)]
#![no_std]

include!(concat!(env!("OUT_DIR"), "/ping.rs"));

use creamy_sdk::{
    api::Plugin,
    bus::{UntypedMessage, buffer::Outgoing},
    declare_plugin,
};

use self::dispatcher::MessageHandler;
use crate::ping::messages::{Ping, Pong};

declare_plugin!(PingPlugin, dispatcher);

struct PingPlugin {
    outgoing: Outgoing,
}

impl Plugin for PingPlugin {
    fn init(outgoing: Outgoing) -> Option<Self> {
        Some(Self { outgoing })
    }
}

impl MessageHandler for PingPlugin {
    #[inline(always)]
    fn handle_ping(&mut self, message: Ping) {
        assert!(
            self.outgoing.send(
                &Pong::PREPARED
                    .with_dst(message.src)
                    .with_serial(message.serial),
            )
        );
    }

    #[inline(always)]
    fn handle_pong(&mut self, message: Pong) {
        assert!(
            self.outgoing.send(
                &Ping::PREPARED
                    .with_dst(message.src)
                    .with_group(1)
                    .with_serial(message.serial),
            )
        );
    }

    #[inline(always)]
    fn handle_unknown_message(
        &mut self,
        _dispatch_value: u32,
        _message: creamy_sdk::bus::UntypedMessage,
    ) {
    }
}
