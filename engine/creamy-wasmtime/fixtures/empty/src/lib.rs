#![no_std]

use creamy_sdk::{Sender, api::Plugin};
use creamy_sdk_wasm::declare_plugin;

use self::generated::dispatcher::{Dispatcher, MessageHandler};

include!(concat!(env!("OUT_DIR"), "/empty.rs"));

declare_plugin!(EmptyPlugin);
struct EmptyPlugin;

impl Plugin for EmptyPlugin {
    type CustomData = ();
    type Delegate = Dispatcher<Self>;
    fn init(_: Sender, _: Self::CustomData) -> Option<Self> {
        Some(Self)
    }
}

impl MessageHandler for EmptyPlugin {}
