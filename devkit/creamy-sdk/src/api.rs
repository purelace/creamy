use cbus_core::{
    UntypedMessage,
    buffer::{Incoming, Outgoing},
};

use crate::dispatcher::MessageHandler;

pub trait Plugin: Sized + CustomHandler {
    fn init(outgoing: Outgoing) -> Option<Self>;
}

pub trait CustomHandler {
    fn handle_message(&mut self, dispatch_value: u32, message: UntypedMessage);
}

const HEADER_MASK: u32 = 0x00_FF_00_FF;
pub fn handle_incoming<H: MessageHandler>(handler: &mut H, mut incoming: Incoming) {
    for &message in incoming.as_slice() {
        let dispatch_value = {
            let message: &UntypedMessage = &message;
            u32::from_le_bytes([message.dst, message.group, message.src, message.kind])
                & HEADER_MASK
        };
        handler.handle_message(dispatch_value, message);
    }

    incoming.clear();
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin:ty, $($dispatcher_module:ident)::+) => {
        static STATE: $crate::spin::Mutex<Option<$crate::state::InnerState<$plugin>>> =
            $crate::spin::Mutex::new(None);

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn init_plugin() -> u8 {
            let outgoing = $crate::get_outgoing();
            if let Some(plugin) = <$plugin as $crate::api::Plugin>::init(outgoing) {
                let state = $crate::state::InnerState::new(plugin);
                let mut lock = STATE.lock();
                *lock = Some(state);

                0
            } else {
                1
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn notify() {
            let incoming = $crate::get_incoming();
            let mut lock = STATE.lock();
            if let Some(ref mut state) = *lock {
                $crate::api::handle_incoming(state, incoming);
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
