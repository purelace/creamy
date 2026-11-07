use cbus_core::{UntypedMessage, buffer::DynIncBuf};

use crate::Sender;

pub trait Plugin: Sized + CustomHandler {
    type CustomData;
    fn init(sender: Sender, data: Self::CustomData) -> Option<Self>;
    fn notify(&mut self) {}
}

impl Plugin for () {
    type CustomData = ();
    fn init(_: Sender, (): Self::CustomData) -> Option<Self> {
        Some(())
    }
}

impl CustomHandler for () {
    fn handle_message(&mut self, _: u32, _: UntypedMessage) {}
    fn handle_on_group_enabled_event(&mut self, _: u8) {}
    fn handle_on_group_disabled_event(&mut self, _: u8) {}
}

pub trait CustomHandler: 'static {
    fn handle_message(&mut self, dispatch_value: u32, message: UntypedMessage);
    fn handle_on_group_enabled_event(&mut self, group_id: u8);
    fn handle_on_group_disabled_event(&mut self, group_id: u8);
}

const HEADER_MASK: u32 = 0x00_FF_00_FF;
pub fn handle_incoming<H: CustomHandler>(handler: &mut H, mut incoming: DynIncBuf) {
    while let Some(message) = incoming.pop() {
        let dispatch_value = {
            let message: &UntypedMessage = &message;
            (u32::from(message.group) << 16 | u32::from(message.kind)) & HEADER_MASK
        };
        handler.handle_message(dispatch_value, message);
    }
}
