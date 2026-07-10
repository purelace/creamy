use alloc::{string::String, vec::Vec};

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
    type Object = String;

    fn write_head(&mut self, object: &Self::Object) -> <Self::Stream as StreamMessage>::Head {
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

    fn write_payload(
        &mut self,
        object: &Self::Object,
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

    fn write_tail(&mut self, object: &Self::Object) -> <Self::Stream as StreamMessage>::Tail {
        let mut data = [0; 27];
        let slice = &object.as_bytes()[self.sent..];
        let to_read = slice.len().min(27);
        data[..to_read].copy_from_slice(&slice[..to_read]);

        LogTail { __unused: 0, data }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        use alloc::string::ToString;
        let string = format_args!($($arg)*);
        let writer = $crate::logging::LogWriter::new($crate::system::builtin::LogType::Info);
        let mut stream = $crate::stream::StreamWriter::new(writer, $crate::stream::StreamId::new(0));
        stream.write(&(string.to_string()));
    };
}

fn test() {
    info!("xdd");
}
