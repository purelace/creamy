use cbus_core::{SubscriberId, buffer::runtime::DynOutBuf};

use crate::{
    message::Message,
    stream::{StreamWriter, StreamWriterFunctions},
};

pub struct Sender {
    buffer: DynOutBuf,
    table: &'static [Option<SubscriberId>; 256],
}

impl Sender {
    #[must_use]
    #[doc(hidden)]
    pub const fn from_static_reference(buffer: DynOutBuf) -> Self {
        Self {
            buffer,
            table: super::state::get_dst_table(),
        }
    }

    /// # Returns
    /// Returns `false` if the buffer is full or a destination point is not set
    pub fn send<T: Message>(&mut self, mut message: T) -> bool {
        let Some(dst) = self.table[T::GROUP.get() as usize] else {
            return false;
        };

        message.with_dst(dst.get());
        message.with_group(T::GROUP.get());
        message.with_kind(T::KIND);

        self.buffer.send(&message)
    }

    pub fn send_to<T: Message>(&mut self, message: &T) -> bool {
        self.buffer.send(message)
    }

    pub fn send_stream<'a, T: StreamWriterFunctions>(
        &mut self,
        mut writer: StreamWriter<T>,
        object: &'a T::Object<'a>,
    ) -> bool {
        let Some(dst) = self.table[T::Stream::GROUP.get() as usize] else {
            return false;
        };
        writer.write(object, dst.get());
        true
    }
}
