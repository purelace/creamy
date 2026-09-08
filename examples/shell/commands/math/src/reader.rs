use alloc::{boxed::Box, vec::Vec};
use core::str::FromStr;

use creamy_sdk::stream::{StreamMessage, StreamReaderFunctions};
use smol_str::SmolStr;

use crate::generated::shell::setup::{AddCommandArgument, SetCommandName};

#[derive(Default)]
pub struct ArgumentReader {
    buffer: Vec<u8>,
}

impl ArgumentReader {
    pub fn into_string(self) -> Result<SmolStr, Box<dyn core::error::Error>> {
        let string = str::from_utf8(&self.buffer)?;
        Ok(SmolStr::from_str(string)?)
    }
}

impl StreamReaderFunctions for ArgumentReader {
    type Stream = AddCommandArgument;

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

#[derive(Default)]
pub struct NameReader {
    buffer: Vec<u8>,
}

impl NameReader {
    pub fn into_string(self) -> Result<SmolStr, Box<dyn core::error::Error>> {
        let string = str::from_utf8(&self.buffer)?;
        Ok(SmolStr::from_str(string)?)
    }
}

impl StreamReaderFunctions for NameReader {
    type Stream = SetCommandName;

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
