use alloc::vec::Vec;
use core::fmt::Arguments;

use crate::{
    stream::{StreamMessage, StreamReaderFunctions, StreamWriterFunctions},
    system::builtin::{
        Log, LogType,
        log::{LogHead, LogPayload, LogTail},
    },
};

pub struct LogReader {
    buffer: Vec<u8>,
}

impl StreamReaderFunctions for LogReader {
    type Stream = Log;

    fn read_single(&mut self, _single: <Self::Stream as StreamMessage>::Payload) {}

    fn read_head(&mut self, head: <Self::Stream as StreamMessage>::Head) {
        if self.buffer.len() < head.length as usize {
            self.buffer = Vec::with_capacity(head.length as usize);
        }

        self.buffer.extend(head.data);
    }

    fn read_payload(&mut self, payload: <Self::Stream as StreamMessage>::Payload) {
        self.buffer.extend(payload.data);
    }

    fn read_tail(&mut self, _tail: <Self::Stream as StreamMessage>::Tail) {}
}

pub struct LogWriter {
    sent: usize,
    kind: LogType,
}

impl LogWriter {
    pub const fn new(kind: LogType) -> Self {
        Self { sent: 0, kind }
    }
}

impl StreamWriterFunctions for LogWriter {
    type Stream = Log;
    type Object<'a> = str;

    fn write_head<'a>(
        &mut self,
        object: &'a Self::Object<'a>,
    ) -> <Self::Stream as StreamMessage>::Head {
        let mut data = [0u8; 20];

        let to_read = object.len().min(20);
        data[..to_read].copy_from_slice(&object.as_bytes()[..to_read]);

        self.sent = to_read;

        LogHead {
            __unused: 0,
            log_type: self.kind,
            __padding0: [0, 0],
            length: self.sent as u32,
            data,
        }
    }

    fn write_payload<'a>(
        &mut self,
        object: &'a Self::Object<'a>,
    ) -> Option<<Self::Stream as StreamMessage>::Payload> {
        if self.sent == object.len() {
            return None;
        }

        const DATA_SIZE: usize = 27;

        let mut data = [0; DATA_SIZE];
        let slice = &object.as_bytes()[self.sent..];
        let to_read = slice.len().min(DATA_SIZE);

        if to_read <= DATA_SIZE {
            // We will send a remainder in tail part
            return None;
        }

        data[..to_read].copy_from_slice(&slice[..to_read]);

        Some(LogPayload { __unused: 0, data })
    }

    fn write_tail<'a>(
        &mut self,
        object: &'a Self::Object<'a>,
    ) -> <Self::Stream as StreamMessage>::Tail {
        let mut data = [0; 27];
        let slice = &object.as_bytes()[self.sent..];
        let to_read = slice.len().min(27);
        data[..to_read].copy_from_slice(&slice[..to_read]);

        LogTail { __unused: 0, data }
    }
}

pub fn send_log(string: Arguments, log_type: LogType) {
    let writer = LogWriter::new(log_type);
    let mut stream = crate::stream::StreamWriter::new(writer, crate::stream::StreamId::new(0));
    if let Some(string) = string.as_str() {
        stream.write(string);
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::logging::LogType::Debug
        );
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::logging::LogType::Info
        );
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::logging::LogType::Warn
        );
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logging::send_log(
            format_args!($($arg)*),
            $crate::logging::LogType::Error
        );
    };
}
