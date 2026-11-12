use alloc::{boxed::Box, string::ToString};
use core::fmt::Arguments;

use cbus_core::SubscriberId;
use creamy_sdk::{
    stream::{StreamId, StreamWriter},
    system::{Log, LogType, log::LogContent},
};

use crate::get_outgoing;

const SYSTEM: SubscriberId = SubscriberId::new_u8(1).unwrap();

pub fn send_log(string: Arguments, log_type: LogType) {
    // TODO: use const
    // TODO: fix allocations
    // TODO: implement dynamic stream id
    let mut stream = StreamWriter::<Log>::new(get_outgoing(), SYSTEM, StreamId::new(0));
    let content = if let Some(string) = string.as_str() {
        Box::from(string.as_bytes())
    } else {
        Box::from(string.to_string().as_bytes())
    };
    stream.send(&LogContent::new(log_type, content));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::sdk::system::LogType::Debug
        )
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::sdk::system::LogType::Info
        )
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::sdk::system::LogType::Warning
        )
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::sdk::system::LogType::Error
        )
    };
}
