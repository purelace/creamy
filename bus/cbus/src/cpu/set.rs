use cbus_core::Subscriber;

use crate::{
    config::BusConfig,
    core::UntypedMessage,
    cpu::{MessagePipeline, PipelineData},
    lookup::LookupTable,
    sys::{SubscriberRange, subscriber::Mask},
};

pub trait InstructionSet<C, S, const M: usize, const CHUNK_SIZE: usize>: Sized
where
    C: BusConfig,
    S: Subscriber,
{
    /// Пишет сообщения в слайс с заданным размером
    fn send_exactly(read: &[UntypedMessage; CHUNK_SIZE], write: &mut [UntypedMessage; CHUNK_SIZE]);

    /// Пишет остаток сообщений в указанный слайс
    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]);

    /// Подготавливает сообщения и пишет в слайс с заданным размером
    fn prepare_and_send_exactly(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        mask: Mask,
        src: u8,
        read: &[UntypedMessage; CHUNK_SIZE],
        write: &mut [UntypedMessage; CHUNK_SIZE],
    );

    /// Подготавливает и пишет остаток сообщений в указанный слайс
    fn prepare_and_send_remainder(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        mask: Mask,
        src: u8,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    );

    /// Подготавливает и пишет остаток сообщений в глобальный буфер
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, name = "bus::prepare_batches", level = "trace")
    )]
    #[inline(always)]
    fn prepare_batches(subscribers: &[u8], data: &mut PipelineData<C, S, M>) {
        let mask = data.memory.subscribers.get_mask();
        for src in subscribers.iter().copied() {
            let sub_data = unsafe { data.memory.subscribers.get_data_mut_unchecked(src) };
            let mut buffer = sub_data.outgoing_mut();

            let write = data.memory.message.reserve_slice(buffer.count() as usize);
            let read = buffer.read_slice();

            Self::slices_prepare_and_send(
                data.lookup_table,
                data.subscriber_range,
                mask,
                src,
                read,
                write,
            );

            buffer.clear();
        }
    }

    /// Читает сообщения из глобального буфера и пишет их в буферы подписчиков
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, name = "bus::send_batches", level = "trace")
    )]
    #[inline(always)]
    fn send_batches(pipeline: &mut MessagePipeline<C, S, M>, data: &mut PipelineData<C, S, M>) {
        let mut batch = core::mem::take(&mut pipeline.batch);

        for (dst, len, ptr_location) in batch.drain(..) {
            let read = data.memory.message.slice(len, ptr_location);

            let sub_data = unsafe { data.memory.subscribers.get_data_mut_unchecked(dst) };
            let mut buffer = sub_data.incoming_mut();
            let write = buffer.write_slice_mut(len);

            Self::slices_send(read, write);

            // SAFETY: TODO
            unsafe {
                buffer.add_count(len);
            }
        }

        let _ = core::mem::replace(&mut pipeline.batch, batch);
    }

    /// Делит оба слайса на равные чанки и передает их в `InstructionSet::send_exaclty`.
    /// Остаток предается в `InstructionSet::send_remainder`
    #[inline(always)]
    fn slices_send(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        let (read_chunks, read_remainder) = read.as_chunks::<CHUNK_SIZE>();
        let (write_chunks, write_remainder) = write.as_chunks_mut::<CHUNK_SIZE>();

        for (read_chunk, write_chunk) in read_chunks.iter().zip(write_chunks) {
            Self::send_exactly(read_chunk, write_chunk);
        }

        Self::send_remainder(read_remainder, write_remainder);
    }

    /// Делит оба слайса на равные чанки и передает их в `InstructionSet::prepare_and_send_exaclty`.
    /// Остаток предается в `InstructionSet::prepare_and_send_remainder`.
    #[inline(always)]
    fn slices_prepare_and_send(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        mask: Mask,
        src: u8,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        let (read_chunks, read_remainder) = read.as_chunks::<CHUNK_SIZE>();
        let (write_chunks, write_remainder) = write.as_chunks_mut::<CHUNK_SIZE>();

        for (read_chunk, write_chunk) in read_chunks.iter().zip(write_chunks) {
            Self::prepare_and_send_exactly(lut, range, mask, src, read_chunk, write_chunk);
        }

        Self::prepare_and_send_remainder(lut, range, mask, src, read_remainder, write_remainder);
    }
}
