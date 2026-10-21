use alloc::{boxed::Box, vec::Vec};
use core::marker::PhantomData;

use super::{StreamInformation, StreamLightweightContent, StreamMessage, StreamType};

#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("Invalid chunk order")]
    InvalidChunkOrder,
    #[error("The reader has finished reading")]
    ReadingFinished,
}

enum ReaderState {
    WaitInformation,
    WaitChunks { buffer: Vec<u8>, additional: usize },
    Finished,
}

pub struct StreamReader<W: StreamMessage> {
    _phantom: PhantomData<W>,
    state: ReaderState,
}

impl<W: StreamMessage> Default for StreamReader<W> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
            state: ReaderState::WaitInformation,
        }
    }
}

impl<W: StreamMessage> StreamReader<W> {
    pub fn accept_message(&mut self, message: &W) -> Option<Result<Converter<W>, ReaderError>> {
        match &mut self.state {
            ReaderState::WaitInformation => {
                let StreamType::Information = message.discriminant() else {
                    return Some(Err(ReaderError::InvalidChunkOrder));
                };

                let info =
                    unsafe { &*core::ptr::from_ref(message.data()).cast::<StreamInformation>() };

                self.state = ReaderState::WaitChunks {
                    // TODO: possible overflow
                    buffer: Vec::with_capacity(
                        core::mem::size_of::<W::Lightweight>() + info.additional_size as usize,
                    ),
                    additional: info.additional_size as usize,
                };
            }
            ReaderState::WaitChunks { buffer, additional } => {
                let StreamType::Chunk = message.discriminant() else {
                    return Some(Err(ReaderError::InvalidChunkOrder));
                };

                let empty_space = buffer.capacity() - buffer.len();
                let to_write = empty_space.min(message.data().len());
                let slice = &message.data()[..to_write];
                buffer.extend_from_slice(slice);

                //TODO: possible overflow
                let expected_size = core::mem::size_of::<W::Lightweight>() + *additional;
                if buffer.len() >= expected_size {
                    let buffer = core::mem::take(buffer).into_boxed_slice();
                    self.state = ReaderState::Finished;
                    return Some(Ok(Converter {
                        buffer,
                        _phantom: PhantomData,
                    }));
                }
            }
            ReaderState::Finished => return Some(Err(ReaderError::ReadingFinished)),
        }

        None
    }
}

#[derive(Debug)]
pub struct Converter<W: StreamMessage> {
    buffer: Box<[u8]>,
    _phantom: PhantomData<W>,
}

impl<W: StreamMessage> Converter<W> {
    pub fn read(&mut self) -> &W::Lightweight {
        W::Lightweight::new(&mut self.buffer)
    }
}
