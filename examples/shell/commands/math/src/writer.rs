use creamy_sdk::stream::{StreamMessage, StreamWriterFunctions};

use crate::generated::shell::setup::{
    RegisterCommand,
    register_command::{RegisterCommandHead, RegisterCommandPayload, RegisterCommandTail},
};

#[derive(Default)]
pub struct StringWriter {
    size: usize,
    sent: usize,
}

impl StreamWriterFunctions for StringWriter {
    type Stream = RegisterCommand;
    type Object<'a> = str;

    fn start<'a>(&mut self, object: &'a Self::Object<'a>) {
        self.size = object.len();
    }

    fn write_head<'a>(
        &mut self,
        object: &'a Self::Object<'a>,
    ) -> <Self::Stream as StreamMessage>::Head {
        let mut data = [0u8; 20];

        let to_read = object.len().min(20);
        data[..to_read].copy_from_slice(&object.as_bytes()[..to_read]);

        self.sent = to_read;

        RegisterCommandHead {
            __unused: 0,
            __padding0: [0; 3],
            length: self.sent as u32,
            data,
        }
    }

    fn write_payload<'a>(
        &mut self,
        object: &'a Self::Object<'a>,
    ) -> Option<<Self::Stream as StreamMessage>::Payload> {
        const DATA_SIZE: usize = 27;

        if self.sent == object.len() {
            return None;
        }

        if object.len() - self.sent <= DATA_SIZE {
            // We will send a remainder in tail part
            return None;
        }

        let mut data = [0; DATA_SIZE];
        let slice = &object.as_bytes()[self.sent..];
        let to_read = slice.len().min(DATA_SIZE);
        self.sent += to_read;

        data[..to_read].copy_from_slice(&slice[..to_read]);

        Some(RegisterCommandPayload { __unused: 0, data })
    }

    fn write_tail<'a>(
        &mut self,
        object: &'a Self::Object<'a>,
    ) -> <Self::Stream as StreamMessage>::Tail {
        let mut data = [0; 27];
        let slice = &object.as_bytes()[self.sent..];
        let to_read = slice.len().min(27);
        data[..to_read].copy_from_slice(&slice[..to_read]);

        RegisterCommandTail { __unused: 0, data }
    }

    fn remaining_length(&self) -> usize {
        self.size - self.sent
    }
}
