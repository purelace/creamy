use cbus_core::{
    UntypedMessage,
    buffer::runtime::{DynIncBuf, DynOutBuf},
};

use crate::dispatcher::MessageHandler;

pub trait Plugin: Sized + CustomHandler {
    fn init(outgoing: DynOutBuf) -> Option<Self>;
}

pub trait CustomHandler {
    fn handle_message(&mut self, dispatch_value: u32, message: UntypedMessage);
}

const HEADER_MASK: u32 = 0x00_FF_00_FF;
pub fn handle_incoming<H: MessageHandler>(handler: &mut H, mut incoming: DynIncBuf) {
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
    ($plugin:ty, $($dispatcher_module:ident)::+) => {
        static mut STATE: Option<$crate::state::InnerState<$plugin>> = None;

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn init() -> u32 {
            unsafe {
                let outgoing = $crate::get_outgoing();
                if let Some(plugin) = <$plugin as $crate::api::Plugin>::init(outgoing) {
                    let instance = $crate::state::InnerState::new(plugin);
                    STATE = Some(instance);

                    0
                } else {
                    1
                }
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn notify() {
            unsafe {
                let incoming = $crate::get_incoming();
                if let Some(state) = STATE.as_mut() {
                    $crate::api::handle_incoming(state, incoming);
                }
            }
        }

        impl $crate::api::CustomHandler for $plugin {
            #[inline(always)]
            fn handle_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
                $($dispatcher_module)::+::dispatch_message(dispatch_value, message, self);
            }
        }
    };
}
