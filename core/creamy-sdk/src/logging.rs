use alloc::{boxed::Box, string::ToString};
use core::fmt::Arguments;

use crate::generated::system::builtin;
pub use crate::generated::system::builtin::{Log, LogType};

pub fn send_log(string: Arguments, log_type: LogType) {
    // TODO: use const
    // TODO: fix allocations
    let mut stream = crate::stream::StreamWriter::<Log>::new(1, crate::stream::StreamId::new(0));
    let content = if let Some(string) = string.as_str() {
        Box::from(string.as_bytes())
    } else {
        Box::from(string.to_string().as_bytes())
    };
    stream.send(&builtin::log::LogContent::new(log_type, content));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::generated::system::builtin::LogType::Debug
        )
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::logging::LogType::Info
        )
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::logging::LogType::Warning
        )
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::logging::LogType::Error
        )
    };
}
