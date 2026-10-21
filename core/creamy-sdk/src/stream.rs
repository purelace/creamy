mod allocator;
mod reader;
mod writer;

use core::fmt::Display;

use creamy_binary::BinaryWriter;
pub use reader::StreamReader;
use thiserror::Error;
pub use writer::StreamWriter;

use crate::message::Message;

//use self::allocator::CacheAllocator;
//static mut STREAM_CACHE_ALLOCATOR: CacheAllocator = CacheAllocator::new();

pub const MAX_STREAM_PAYLOAD: usize = 27;
pub const STREAM_TYPE_MASK: u8 = 0b_1000_0000;
pub const STREAM_ID_MASK: u8 = 0b_0111_1111;

#[derive(Clone, Copy)]
pub struct Array(u32);
impl Array {
    #[must_use]
    pub const fn count(self) -> u32 {
        self.0
    }
}

pub trait StreamContent: BinaryWriter {
    type Message: StreamMessage;

    fn additional_data(&self) -> u32;
}

pub trait StreamLightweightContent: Sized {
    fn new(data: &mut [u8]) -> &Self;
}

pub trait StreamMessage: Message {
    const PREPARED: Self;
    const TIMEOUT: u8;
    type Content: StreamContent;
    type Lightweight: StreamLightweightContent;

    fn data(&self) -> &[u8; MAX_STREAM_PAYLOAD];
    fn data_mut(&mut self) -> &mut [u8; MAX_STREAM_PAYLOAD];

    fn with_stream_id(&mut self, value: StreamId) -> &mut Self;
    fn stream_id(&self) -> StreamId;

    fn with_discriminant(&mut self, value: StreamType) -> &mut Self;
    fn discriminant(&self) -> StreamType;
}

#[repr(C, packed)]
pub struct StreamInformation {
    additional_size: u32,
    _padding: [u8; 23],
}

impl StreamInformation {
    #[must_use]
    pub const fn new(additional_size: u32) -> Self {
        Self {
            additional_size,
            _padding: [0; 23],
        }
    }

    #[must_use]
    pub const fn additional_size(&self) -> u32 {
        self.additional_size
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StreamType {
    Information = 0,
    Chunk = 1,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamId(u8);
impl StreamId {
    const MAX_VALUES: u8 = 128;

    #[must_use]
    pub const fn new(id: u8) -> Self {
        assert!(id <= Self::MAX_VALUES);
        Self(id)
    }

    #[must_use]
    pub const fn new_trim(id: u8) -> Self {
        Self(id & STREAM_ID_MASK)
    }

    #[must_use]
    pub const fn safe_new(id: u8) -> Option<Self> {
        if id > Self::MAX_VALUES {
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
        self.0 > Self::MAX_VALUES
    }
}

impl Display for StreamId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamReaderError {
    #[error("stream {stream_id} timed out while waiting for {expected:?} chunk")]
    Timeout {
        stream_id: StreamId,
        expected: StreamType,
    },

    #[error("stream {stream_id} expected {expected:?} chunk, got {got:?}")]
    UnexpectedDiscriminant {
        stream_id: StreamId,
        expected: StreamType,
        got: StreamType,
    },

    #[error("stream {stream_id} is already closed")]
    StreamAlreadyClosed { stream_id: StreamId },
}
