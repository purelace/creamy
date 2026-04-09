use crate::{
    core::UntypedMessage,
    cpu::{MessagePipeline, PipelineData},
    lookup::LookupTable,
};

pub trait InstructionSet<const CHUNK_SIZE: usize>: Sized {
    /// Пишет сообщения в слайс с заданным размером
    fn send_exactly(read: &[UntypedMessage; CHUNK_SIZE], write: &mut [UntypedMessage; CHUNK_SIZE]);

    /// Пишет остаток сообщений в указанный слайс
    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]);

    /// Подготавливает сообщения и пишет в слайс с заданным размером
    fn prepare_and_send_exactly(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage; CHUNK_SIZE],
        write: &mut [UntypedMessage; CHUNK_SIZE],
    );

    /// Подготавливает и пишет остаток сообщений в указанный слайс
    fn prepare_and_send_remainder(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    );

    /// Подготавливает и пишет остаток сообщений в глобальный буфер
    #[inline(always)]
    fn prepare_batches(subscribers: &[u8], data: &mut PipelineData) {
        let capacity = data.memory.read.slice_capacity();
        for src in subscribers.iter().copied() {
            let src = src as usize;
            let header = data.memory.read.header_for(src);
            let read = header.read_slice(capacity);
            let write = data.memory.message.reserve_slice(header.count as usize);

            Self::slices_prepare_and_send(data.lookup_table, src, read, write);

            header.count = 0;
        }
    }

    /// Читает сообщения из глобального буфера и пишет их в буферы подписчиков
    #[inline(always)]
    fn send_batches(pipeline: &mut MessagePipeline, data: &mut PipelineData) {
        let mut batch = std::mem::take(&mut pipeline.batch);

        for (dst, len, ptr_location) in batch.drain(..) {
            let read = data.memory.message.slice(len as usize, ptr_location);
            let header = data.memory.write.header_for(dst as usize);
            let write = header.write_slice_mut(len as usize);

            Self::slices_send(read, write);

            header.count += len;
        }

        let _ = std::mem::replace(&mut pipeline.batch, batch);
    }

    /// Делит оба слайса на равные чанки и передает их в `InstructionSet::send_exaclty`.
    /// Остаток предается в `InstructionSet::send_remainder`
    #[inline(always)]
    fn slices_send(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        let mut read_chunks = read.chunks_exact(CHUNK_SIZE);
        let mut write_chunks = write.chunks_exact_mut(CHUNK_SIZE);

        for (read_chunk, write_chunk) in (&mut read_chunks).zip(&mut write_chunks) {
            let read_slice: &[UntypedMessage; CHUNK_SIZE] = read_chunk.try_into().unwrap();
            let write_slice: &mut [UntypedMessage; CHUNK_SIZE] = write_chunk.try_into().unwrap();

            Self::send_exactly(read_slice, write_slice);
        }

        let read = read_chunks.remainder();
        let write = write_chunks.into_remainder();
        Self::send_remainder(read, write);
    }

    /// Делит оба слайса на равные чанки и передает их в `InstructionSet::prepare_and_send_exaclty`.
    /// Остаток предается в `InstructionSet::prepare_and_send_remainder`.
    #[inline(always)]
    fn slices_prepare_and_send(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        let mut read_chunks = read.chunks_exact(CHUNK_SIZE);
        let mut write_chunks = write.chunks_exact_mut(CHUNK_SIZE);

        for (read_chunk, write_chunk) in (&mut read_chunks).zip(&mut write_chunks) {
            let read_slice: &[UntypedMessage; CHUNK_SIZE] = read_chunk.try_into().unwrap();
            let write_slice: &mut [UntypedMessage; CHUNK_SIZE] = write_chunk.try_into().unwrap();

            Self::prepare_and_send_exactly(lut, src, read_slice, write_slice);
        }

        let read = read_chunks.remainder();
        let write = write_chunks.into_remainder();
        Self::prepare_and_send_remainder(lut, src, read, write);
    }
}
