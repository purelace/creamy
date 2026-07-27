#![allow(clippy::inline_always)]
#![no_std]

include!(concat!(env!("OUT_DIR"), "/ping.rs"));

use creamy_sdk::{
    api::Plugin,
    bus::{UntypedMessage, buffer::runtime::DynOutBuf},
    declare_plugin, error, info, warn,
};

use self::dispatcher::MessageHandler;
use crate::ping::messages::{Ping, Pong};

declare_plugin!(PingPlugin, dispatcher);

struct PingPlugin {
    outgoing: DynOutBuf,
}

impl Plugin for PingPlugin {
    fn init(outgoing: DynOutBuf) -> Option<Self> {
        info!("Hello, World!");
        warn!("Дарова, заебал!");
        error!("Как дела?");
        Some(Self { outgoing })
    }

    fn notify(&mut self) {
        info!("Йо-йо-йо 1-4-8-3 да 3-6-9, Альбукерке жжёт, чё-кого, сучара, жди сигнала");
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
