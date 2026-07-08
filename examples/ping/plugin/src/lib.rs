#![no_std]

mod protocols;

use creamy_sdk::{
    api::{Plugin, PluginState},
    bus::buffer::{Incoming, Outgoing},
    declare_plugin,
};

use crate::protocols::ping::messages::{Ping, Pong};

declare_plugin!(PingState, PingPlugin);

struct PingState;
impl PluginState for PingState {
    fn init() -> Self {
        Self
    }
}

struct PingPlugin {}
impl Plugin<PingState> for PingPlugin {
    fn init(state: PingState) -> u8 {
        0
    }

    fn notify(mut incoming: Incoming, mut outgoing: Outgoing) {
        for message in incoming.as_slice() {
            if message.group != 1 {
                continue;
            }

            let message = message.cast::<Ping>();
            outgoing.send(
                &Pong::PREPARED
                    .with_dst(message.src)
                    .with_group(1)
                    .with_serial(message.serial),
            );
        }

        incoming.clear();
    }
}
