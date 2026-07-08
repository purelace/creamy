use core::fmt::Display;

use cbus_core::message::TypedMessage;
use thiserror::Error;

use crate::utils::extract_payload;

pub const MAX_STREAM_PAYLOAD: usize = 28;

pub trait StreamHead {}
pub trait StreamPayload {}
pub trait StreamTail {}

impl StreamHead for () {}
impl StreamTail for () {}

pub trait StreamMessage: TypedMessage {
    const TIMEOUT: u8;
    type Head: StreamHead;
    type Payload: StreamPayload;
    type Tail: StreamTail;

    fn stream_id(&self) -> StreamId;
    fn discriminant(&self) -> StreamChunkType;
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StreamChunkType {
    Single = 0,
    Head = 1,
    Payload = 2,
    Tail = 3,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamId(u8);
impl StreamId {
    #[must_use]
    pub const fn new(id: u8) -> Self {
        assert!(id <= 64);
        Self(id)
    }

    #[must_use]
    pub const fn new_trim(id: u8) -> Self {
        Self(id & 0b_0011_1111)
    }

    #[must_use]
    pub const fn safe_new(id: u8) -> Option<Self> {
        if id > 64 {
            return None;
        }

        Some(Self(id))
    }

    #[must_use]
    pub const fn value(&self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn is_broken(&self) -> bool {
        self.0 > 64
    }
}

impl Display for StreamId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.value())
    }
}

pub struct StreamReader<R: StreamReaderFunctions> {
    reader: R,
    frame: u8,
    id: StreamId,
    state: StreamChunkType,
}

pub trait StreamReaderFunctions {
    type Stream: StreamMessage;

    fn read_single(&mut self, single: <Self::Stream as StreamMessage>::Payload);
    fn read_head(&mut self, head: <Self::Stream as StreamMessage>::Head);
    fn read_payload(&mut self, payload: <Self::Stream as StreamMessage>::Payload);
    fn read_tail(&mut self, tail: <Self::Stream as StreamMessage>::Tail);
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamReaderError {
    #[error("stream {stream_id} timed out while waiting for {expected:?} chunk")]
    Timeout {
        stream_id: StreamId,
        expected: StreamChunkType,
    },

    #[error("stream {stream_id} expected {expected:?} chunk, got {got:?}")]
    UnexpectedDiscriminant {
        stream_id: StreamId,
        expected: StreamChunkType,
        got: StreamChunkType,
    },

    #[error("stream {stream_id} is already closed")]
    StreamAlreadyClosed { stream_id: StreamId },
}

impl<R: StreamReaderFunctions> StreamReader<R> {
    pub const fn new(id: StreamId, reader: R) -> Self {
        Self {
            reader,
            frame: 0,
            id,
            state: StreamChunkType::Head,
        }
    }

    /// Возвращает true когда чтение потока было полностью завершено
    pub fn read(&mut self, message: R::Stream) -> Result<bool, StreamReaderError> {
        match self.state {
            StreamChunkType::Single => {
                todo!();
            }
            StreamChunkType::Head => {
                if let StreamChunkType::Head = message.discriminant() {
                    self.state = StreamChunkType::Payload;
                } else {
                    return Err(StreamReaderError::UnexpectedDiscriminant {
                        stream_id: self.id,
                        expected: StreamChunkType::Head,
                        got: message.discriminant(),
                    });
                }

                let head =
                    extract_payload::<R::Stream, <R::Stream as StreamMessage>::Head>(&message);
                self.reader.read_head(head);

                Ok(false)
            }
            StreamChunkType::Payload => {
                match message.discriminant() {
                    StreamChunkType::Payload => {
                        self.state = StreamChunkType::Payload;
                        let payload = extract_payload::<
                            R::Stream,
                            <R::Stream as StreamMessage>::Payload,
                        >(&message);
                        self.reader.read_payload(payload);
                    }
                    StreamChunkType::Tail => {
                        self.state = StreamChunkType::Tail;
                        let tail = extract_payload::<R::Stream, <R::Stream as StreamMessage>::Tail>(
                            &message,
                        );
                        self.reader.read_tail(tail);
                        return Ok(true);
                    }
                    _ => {
                        return Err(StreamReaderError::UnexpectedDiscriminant {
                            stream_id: self.id,
                            expected: StreamChunkType::Tail, //Or Payload
                            got: message.discriminant(),
                        });
                    }
                }

                Ok(false)
            }
            StreamChunkType::Tail => {
                Err(StreamReaderError::StreamAlreadyClosed { stream_id: self.id })
            }
        }
    }

    pub const fn tick(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    pub const fn is_timed_out(&self) -> bool {
        self.frame >= <R::Stream as StreamMessage>::TIMEOUT
    }

    pub const fn reset_frame_timer(&mut self) {
        self.frame = 0;
    }

    pub fn into_reader(self) -> R {
        self.reader
    }
}
