use cbus_core::{
    UntypedMessage,
    buffer::{Incoming, Outgoing},
};

const HEADER_MASK: u32 = 0x00_FF_00_FF;

#[inline]
const fn get_dispatch_value(message: &UntypedMessage) -> u32 {
    u32::from_le_bytes([message.dst, message.group, message.src, message.kind]) & HEADER_MASK
}

//TODO: static dispatcher
pub fn handle_messages(mut incoming: Incoming) {
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

pub fn handle_message(message: UntypedMessage) {}

#[macro_export]
macro_rules! declare_plugin {
    ($s:ty, $t:ty) => {
        #[unsafe(no_mangle)]
        #[allow(clippy::missing_safety_doc)]
        pub unsafe extern "C" fn init_plugin() -> u8 {
            let state = <$s>::init();
            <$t as $crate::api::Plugin<$s>>::init(state)
        }

        #[unsafe(no_mangle)]
        #[allow(clippy::missing_safety_doc)]
        pub unsafe extern "C" fn notify() {
            let incoming = $crate::get_incoming();
            let outgoing = $crate::get_outgoing();
            <$t as $crate::api::Plugin<$s>>::notify(incoming, outgoing);
        }
    };
}

pub trait PluginState {
    fn init() -> Self;
}

pub trait Plugin<S: PluginState> {
    fn init(state: S) -> u8;
    fn notify(incoming: Incoming, outgoing: Outgoing);
}
