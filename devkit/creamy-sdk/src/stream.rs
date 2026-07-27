use alloc::boxed::Box;
use core::{any::TypeId, fmt::Display};

use cbus_core::message::TypedMessage;
use downcast_rs::Downcast;
use rustc_hash::FxHashMap;
use thiserror::Error;

use crate::{get_outgoing, system::builtin::Log, utils::extract_payload};

pub const MAX_STREAM_PAYLOAD: usize = 28;

pub trait StreamData {
    fn cast_to_array(self) -> [u8; 28];
}

impl StreamData for () {
    fn cast_to_array(self) -> [u8; 28] {
        [0; 28]
    }
}

pub trait StreamHead: StreamData {}
pub trait StreamPayload: StreamData {}
pub trait StreamTail: StreamData {}

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

pub trait StreamReaderFunctions {
    type Stream: StreamMessage;

    fn read_single(&mut self, single: <Self::Stream as StreamMessage>::Payload);
    fn read_head(&mut self, head: <Self::Stream as StreamMessage>::Head);
    fn read_payload(&mut self, payload: <Self::Stream as StreamMessage>::Payload);
    fn read_tail(&mut self, tail: <Self::Stream as StreamMessage>::Tail);
}

pub trait StreamWriterFunctions {
    type Stream: StreamMessage;
    type Object<'a>: ?Sized
    where
        Self: 'a;

    fn start<'a>(&mut self, object: &'a Self::Object<'a>);

    fn write_head<'a>(
        &mut self,
        object: &'a Self::Object<'a>,
    ) -> <Self::Stream as StreamMessage>::Head;

    fn write_payload<'a>(
        &mut self,
        object: &'a Self::Object<'a>,
    ) -> Option<<Self::Stream as StreamMessage>::Payload>;

    fn write_tail<'a>(
        &mut self,
        object: &'a Self::Object<'a>,
    ) -> <Self::Stream as StreamMessage>::Tail;

    fn remaining_length(&self) -> usize;
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

pub struct StreamReader<R: StreamReaderFunctions> {
    reader: R,
    frame: u8,
    id: StreamId,
    state: StreamChunkType,
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
        let d = message.discriminant();
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

pub struct StreamWriter<W: StreamWriterFunctions> {
    writer: W,
    frame: u8,
    id: StreamId,
}

impl<W: StreamWriterFunctions> StreamWriter<W> {
    #[must_use]
    pub const fn new(writer: W, id: StreamId) -> Self {
        Self {
            writer,
            frame: 0,
            id,
        }
    }

    pub fn write<'a>(&mut self, object: &'a W::Object<'a>) {
        let mut outgoing = get_outgoing();

        let mut message = Log::PREPARED;
        message.dst = 1;
        message.with_stream_id(self.id);

        let mut write_and_send = |data: [u8; 28], state: StreamChunkType| {
            let data = data[1..].try_into().unwrap();
            message.data = data;
            message.with_discriminant(state);
            assert!(outgoing.send(&message));
        };

        self.writer.start(object);

        let head = self.writer.write_head(object);
        let data = head.cast_to_array();
        write_and_send(data, StreamChunkType::Head);

        loop {
            if let Some(payload) = self.writer.write_payload(object) {
                let data = payload.cast_to_array();
                write_and_send(data, StreamChunkType::Payload);
            } else {
                let tail = self.writer.write_tail(object);
                let data = tail.cast_to_array();
                write_and_send(data, StreamChunkType::Tail);
                break;
            }
        }
    }

    pub const fn tick(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    pub const fn is_timed_out(&self) -> bool {
        self.frame >= <W::Stream as StreamMessage>::TIMEOUT
    }

    pub const fn reset_frame_timer(&mut self) {
        self.frame = 0;
    }

    pub fn into_writer(self) -> W {
        self.writer
    }
}

pub trait StreamStorage {
    fn get_or_add_writer(&mut self, writer: dyn StreamWriterMarker);
    fn get_or_add_reader(&mut self, reader: dyn StreamReaderMarker);
}

pub struct TypedStreamStorage<R: StreamReaderFunctions, W: StreamWriterFunctions> {
    readers: FxHashMap<u16, StreamReader<R>>,
    writers: FxHashMap<u16, StreamWriter<W>>,
}

pub struct StreamStorages {
    inner: FxHashMap<TypeId, Box<dyn StreamStorage>>,
}

impl StreamStorages {
    pub fn read_message(&mut self, message: impl StreamMessage) {}
}

pub trait StreamReaderMarker: Downcast {}
downcast_rs::impl_downcast!(StreamReaderMarker);

pub trait StreamWriterMarker: Downcast {}
downcast_rs::impl_downcast!(StreamWriterMarker);

impl StreamReaderFunctions for () {
    type Stream = Log;

    fn read_single(&mut self, single: <Self::Stream as StreamMessage>::Payload) {}
    fn read_head(&mut self, head: <Self::Stream as StreamMessage>::Head) {}
    fn read_payload(&mut self, payload: <Self::Stream as StreamMessage>::Payload) {}
    fn read_tail(&mut self, tail: <Self::Stream as StreamMessage>::Tail) {}
}

//static STREAMS: [StreamReader<()>; 64] = [StreamReader::new(StreamId::new(0), ()); 64];
