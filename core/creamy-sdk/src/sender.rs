use cbus_core::buffer::DynOutBuf;

use crate::{
    message::Message,
    state::Table,
    stream::{StreamContent, StreamId, StreamMessage, StreamWriter},
};

pub struct Sender {
    buffer: DynOutBuf,
    table: *const Table,
}

impl Sender {
    pub(crate) const fn new(buffer: DynOutBuf, table: *const Table) -> Self {
        Self { buffer, table }
    }

    /// # Returns
    /// Returns `false` if the buffer is full or a destination point is not set
    pub fn send<T: Message>(&mut self, mut message: T) -> bool {
        cfg_select! {
            debug_assertions => {
                #[allow(clippy::missing_panics_doc)]
                let table = unsafe {
                    self.table
                        .as_ref()
                        .expect("cast pointer to shared reference bug")
                };
            }
            _ => {
                let table = unsafe { self.table.as_ref_unchecked() };
            }
        }
        let slice = table.get(T::PROTOCOL);
        let mut success = !slice.is_empty();
        for dst in slice {
            message.with_dst(*dst);
            message.with_protocol(T::PROTOCOL);
            message.with_kind(T::KIND);

            success &= self.buffer.send(&message);
        }

        success
    }

    #[must_use]
    pub const fn send_to<T: Message>(&mut self, message: &T) -> bool {
        self.buffer.send(message)
    }

    pub fn send_stream<C>(&mut self, content: &C) -> bool
    where
        C: StreamContent,
        C::Message: StreamMessage<Content = C>,
    {
        cfg_select! {
            debug_assertions => {
                #[allow(clippy::missing_panics_doc)]
                let table = unsafe {
                    self.table
                        .as_ref()
                        .expect("cast pointer to shared reference bug")
                };
            }
            _ => {
                let table = unsafe { self.table.as_ref_unchecked() };
            }
        }
        let slice = table.get(<C::Message>::PROTOCOL);
        if slice.is_empty() {
            return false;
        }

        for dst in slice {
            //TODO: dynamic stream id
            let mut writer =
                StreamWriter::<C::Message>::new(self.buffer.clone(), *dst, StreamId::new(10));
            writer.send(content);
        }

        true
    }
}
