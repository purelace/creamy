use cbus_core::buffer::{Incoming, Outgoing};

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
