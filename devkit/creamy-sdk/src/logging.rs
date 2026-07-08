use alloc::vec::Vec;

use crate::{
    get_outgoing,
    stream::{StreamMessage, StreamReaderFunctions},
    system::builtin::{LogDebug, LogInfo},
};

const fn empty() -> LogInfo {
    LogInfo {
        dst: 0,
        group: 0,
        src: 0,
        kind: LogInfo::KIND,
        meta: 0,
        data: [0u8; 27],
    }
}

/*
fn write_log(data: impl AsRef<[u8]>) {
    let mut out = get_outgoing();
    let mut message = LogInfo {
        dst: 0,
        group: 0,
        src: 0,
        kind: LogInfo::KIND,
        meta: 0,
        data: [0u8; 27],
    };
    message.with_stream_id(StreamId::new_trim(1));

    let data = data.as_ref();
    let len = data.len();

    let len_bytes = len.to_le_bytes();
    for (idx, byte) in len_bytes.into_iter().enumerate() {
        message.data[idx] = byte;
    }

    if len <= 27 - 4 {
        for (idx, byte) in data.iter().enumerate() {
            message.data[idx + 4] = *byte;
        }
        message.with_discriminant(StreamChunkType::Single);
        out.send_many_iter_exact(core::iter::once(message));
        return;
    }

    {
        let first_part = data.as_array::<23>().unwrap();
        for (idx, byte) in first_part.iter().enumerate() {
            message.data[idx + 4] = *byte;
        }
        message.with_discriminant(StreamChunkType::Payload);
        out.send_many_iter_exact(core::iter::once(message));
    }

    let (chunks, remainder) = data[23..].as_chunks::<27>();
    for chunk in chunks {
        for (idx, byte) in chunk.iter().enumerate() {
            message.data[idx] = *byte;
        }
        message.with_discriminant(StreamChunkType::Payload);
        out.send_many_iter_exact(core::iter::once(message));
    }

    for (idx, byte) in remainder.iter().enumerate() {
        message.data[idx] = *byte;
    }

    message.with_discriminant(StreamChunkType::Tail);
    out.send_many_iter_exact(core::iter::once(message));
}
*/
pub fn info(message: &str) {
    let out = get_outgoing();
    //assert!(LogInfo {
    //    dst: todo!(),
    //    group: todo!(),
    //    src: todo!(),
    //    kind: todo!(),
    //    meta: todo!(),
    //    data: todo!(),
    //});
}

pub struct LogReader {
    buffer: Vec<u8>,
}

impl StreamReaderFunctions for LogReader {
    type Stream = LogDebug;

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
