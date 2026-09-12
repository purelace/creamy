use cbus_core::{UntypedMessage, buffer::runtime::DynIncBuf};

use crate::Sender;

pub trait Plugin: Sized + CustomHandler {
    fn init(sender: Sender) -> Option<Self>;
    fn notify(&mut self) {}
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

#[macro_export]
macro_rules! declare_plugin {
    (
        $plugin:ty,
        $generated_module:ident
    ) => {
        #[doc(hidden)]
        static mut STATE: Option<$crate::state::InnerState<$plugin>> = None;

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn init() -> u32 {
            unsafe {
                let sender = $crate::Sender::from_static_reference($crate::get_outgoing());
                if let Some(plugin) = <$plugin as $crate::api::Plugin>::init(sender) {
                    let instance = $crate::state::InnerState::new(plugin);
                    STATE = Some(instance);

                    0
                } else {
                    1
                }
            }
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn notify() {
            unsafe {
                let incoming = $crate::get_incoming();
                if let Some(state) = STATE.as_mut() {
                    $crate::api::handle_incoming(state, incoming);
                    state.notify();
                }
            }
        }

        #[doc(hidden)]
        impl $crate::api::CustomHandler for $plugin {
            fn handle_message(
                &mut self,
                dispatch_value: u32,
                message: $crate::message::UntypedMessage,
            ) {
                $generated_module::dispatcher::dispatch_message(dispatch_value, message, self);
            }

            fn handle_on_group_enabled_event(&mut self, group_id: u8) {
                $generated_module::dispatcher::handle_on_group_enabled_event(group_id, self);
            }

            fn handle_on_group_disabled_event(&mut self, group_id: u8) {
                $generated_module::dispatcher::handle_on_group_disabled_event(group_id, self);
            }
        }
    };
}
