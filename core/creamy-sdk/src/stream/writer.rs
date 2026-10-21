use creamy_binary::{BinaryWriter, ErrorType, Write};

use super::{MAX_STREAM_PAYLOAD, StreamContent, StreamId, StreamMessage, StreamType};
use crate::get_outgoing;

pub struct StreamWriter<W: StreamMessage> {
    message: W,
    frame: u8,
    bytes_written: usize,
    //id: StreamId,
}

impl<W: StreamMessage> ErrorType for StreamWriter<W> {
    type Error = !;
}

impl<W: StreamMessage> StreamWriter<W> {
    #[must_use]
    pub fn new(dst: u8, stream_id: StreamId) -> Self {
        let mut message = W::PREPARED;
        message.with_dst(dst);
        message.with_stream_id(stream_id);
        Self {
            message,
            frame: 0,
            bytes_written: 0,
        }
    }

    pub fn send(&mut self, content: &W::Content) {
        self.message.with_discriminant(StreamType::Information);
        let bytes = content.additional_data().to_ne_bytes();
        let start = &mut self.message.data_mut()[..4];
        start.copy_from_slice(&bytes);
        self.send_message();

        content.write_ne(self).unwrap();
        self.flush_packet();
    }

    fn send_message(&self) {
        let mut outgoing = get_outgoing();
        assert!(outgoing.send(&self.message));
    }

    fn flush_packet(&mut self) {
        if self.bytes_written == 0 {
            return;
        }

        self.send_message();
        self.bytes_written = 0;
    }

    pub const fn tick(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    pub const fn is_timed_out(&self) -> bool {
        self.frame >= W::TIMEOUT
    }

    pub const fn reset_frame_timer(&mut self) {
        self.frame = 0;
    }
}

impl<W: StreamMessage> Write for StreamWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.message.with_discriminant(StreamType::Chunk);
        for &byte in buf {
            self.message.data_mut()[self.bytes_written] = byte;
            self.bytes_written += 1;

            if self.bytes_written == MAX_STREAM_PAYLOAD {
                self.send_message();
                self.bytes_written = 0;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
