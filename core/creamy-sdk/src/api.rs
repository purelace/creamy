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
    fn handle_on_protocol_enabled_event(protocol_id: u8, handler: &mut Self::Handler);
    fn handle_on_protocol_disabled_event(protocol_id: u8, handler: &mut Self::Handler);
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl DispatchDelegate for () {
    type Handler = ();
    fn dispatch_message(_: u32, _: UntypedMessage, (): &mut Self::Handler) {}
    fn handle_on_protocol_enabled_event(_: u8, (): &mut Self::Handler) {}
    fn handle_on_protocol_disabled_event(_: u8, (): &mut Self::Handler) {}
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Plugin for () {
    type CustomData = ();
    type Delegate = ();
    fn init(_: Sender, (): Self::CustomData) -> Option<Self> {
        Some(())
    }
}
