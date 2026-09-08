use creamy::sdk::stream::{StreamMessage, StreamReaderFunctions};

use crate::generated::shell::setup::RegisterCommand;

#[derive(Default)]
pub struct RegisterCommandReader {
    buffer: Vec<u8>,
}

impl RegisterCommandReader {
    pub fn into_string(self) -> Result<String, Box<dyn core::error::Error>> {
        Ok(str::from_utf8(&self.buffer)?.to_string())
    }
}

impl StreamReaderFunctions for RegisterCommandReader {
    type Stream = RegisterCommand;

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

    fn read_tail(&mut self, tail: <Self::Stream as StreamMessage>::Tail) {
        self.buffer.extend(tail.data);
    }
}
