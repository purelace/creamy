use cbus_core::Subscriber;

use crate::{
    config::BusConfig,
    core::UntypedMessage,
    cpu::{MessagePipeline, PipelineData},
    lookup::LookupTable,
};

pub trait InstructionSet<C, S, const M: usize, const CHUNK_SIZE: usize>: Sized
where
    C: BusConfig,
    S: Subscriber,
{
    #[allow(dead_code)]
    const CHUNK_SIZE: usize = CHUNK_SIZE;
    /// Пишет сообщения в слайс с заданным размером
    fn send_exactly(read: &[UntypedMessage; CHUNK_SIZE], write: &mut [UntypedMessage; CHUNK_SIZE]);

    /// Пишет остаток сообщений в указанный слайс
    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]);

    /// Подготавливает сообщения и пишет в слайс с заданным размером
    fn prepare_and_send_exactly(
        lut: &LookupTable<C>,
        src: usize,
        read: &[UntypedMessage; CHUNK_SIZE],
        write: &mut [UntypedMessage; CHUNK_SIZE],
    );

    /// Подготавливает и пишет остаток сообщений в указанный слайс
    fn prepare_and_send_remainder(
        lut: &LookupTable<C>,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    );

    /// Подготавливает и пишет остаток сообщений в глобальный буфер
    #[tracing::instrument(skip_all)]
    #[inline(always)]
    fn prepare_batches(subscribers: &[u8], data: &mut PipelineData<C, S, M>) {
        for src in subscribers.iter().copied() {
            let src = src as usize;

            let sub_data = &mut data.memory.subscribers[src];
            let mut buffer = sub_data.outgoing_mut();

            let write = data.memory.message.reserve_slice(buffer.count() as usize);
            let read = buffer.read_slice();

            Self::slices_prepare_and_send(data.lookup_table, src, read, write);

            buffer.set_count(0);
        }
    }

    /// Читает сообщения из глобального буфера и пишет их в буферы подписчиков
    #[tracing::instrument(skip_all)]
    #[inline(always)]
    fn send_batches(pipeline: &mut MessagePipeline<C, S, M>, data: &mut PipelineData<C, S, M>) {
        let mut batch = core::mem::take(&mut pipeline.batch);

        for (dst, len, ptr_location) in batch.drain(..) {
            let read = data.memory.message.slice(len as usize, ptr_location);

            let sub_data = &mut data.memory.subscribers[dst as usize];
            let mut buffer = sub_data.incoming_mut();
            let write = buffer.write_slice_mut(len as usize);

            Self::slices_send(read, write);

            buffer.add_count(len);
        }

        let _ = core::mem::replace(&mut pipeline.batch, batch);
    }

    /// Делит оба слайса на равные чанки и передает их в `InstructionSet::send_exaclty`.
    /// Остаток предается в `InstructionSet::send_remainder`
    #[tracing::instrument(skip_all)]
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
    #[tracing::instrument(skip_all)]
    #[inline(always)]
    fn slices_prepare_and_send(
        lut: &LookupTable<C>,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        tracing::info!("to write: {}", write.len());

        let (read_chunks, read_remainder) = read.as_chunks::<CHUNK_SIZE>();
        let (write_chunks, write_remainder) = write.as_chunks_mut::<CHUNK_SIZE>();

        for (read_chunk, write_chunk) in read_chunks.iter().zip(write_chunks) {
            Self::prepare_and_send_exactly(lut, src, read_chunk, write_chunk);
        }

        Self::prepare_and_send_remainder(lut, src, read_remainder, write_remainder);
    }
}
