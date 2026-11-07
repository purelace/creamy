use cbus_core::UntypedMessage;

use crate::Sender;

pub trait Plugin: Sized + 'static {
    type CustomData;
    type Delegate: DispatchDelegate<Handler = Self>;
    fn init(sender: Sender, data: Self::CustomData) -> Option<Self>;
    fn notify(&mut self) {}
}

pub trait DispatchDelegate {
    type Handler;
    fn dispatch_message(dispatch_value: u32, message: UntypedMessage, handler: &mut Self::Handler);
    fn handle_on_group_enabled_event(group_id: u8, handler: &mut Self::Handler);
    fn handle_on_group_disable_event(group_id: u8, handler: &mut Self::Handler);
}

impl DispatchDelegate for () {
    type Handler = ();
    fn dispatch_message(_: u32, _: UntypedMessage, _: &mut Self::Handler) {}
    fn handle_on_group_enabled_event(_: u8, _: &mut Self::Handler) {}
    fn handle_on_group_disable_event(_: u8, _: &mut Self::Handler) {}
}

impl Plugin for () {
    type CustomData = ();
    type Delegate = ();
    fn init(_: Sender, (): Self::CustomData) -> Option<Self> {
        Some(())
    }
}
