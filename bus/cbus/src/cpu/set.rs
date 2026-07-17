use crate::{
    core::UntypedMessage,
    cpu::{MessagePipeline, PipelineData},
    lookup::LookupTable,
    sys::Header,
};

pub trait InstructionSet<const CHUNK_SIZE: usize>: Sized {
    #[allow(dead_code)]
    const CHUNK_SIZE: usize = CHUNK_SIZE;
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

            let header = data.memory.read.header_mut_ptr_for(src);
            let read = Header::read_slice_mut_test(header, capacity);

            unsafe {
                let write = data.memory.message.reserve_slice((*header).count as usize);

                Self::slices_prepare_and_send(data.lookup_table, src, read, write);
                Header::set_count(header, 0);
            }
        }
    }

    /// Читает сообщения из глобального буфера и пишет их в буферы подписчиков
    #[inline(always)]
    fn send_batches(pipeline: &mut MessagePipeline, data: &mut PipelineData) {
        let mut batch = core::mem::take(&mut pipeline.batch);

        for (dst, len, ptr_location) in batch.drain(..) {
            let read = data.memory.message.slice(len as usize, ptr_location);
            let header = data.memory.write.header_mut_ptr_for(dst as usize);
            let write = Header::write_slice_mut(header, len as usize);

            Self::slices_send(read, write);

            unsafe {
                (*header).count += len;
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
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        let (read_chunks, read_remainder) = read.as_chunks::<CHUNK_SIZE>();
        let (write_chunks, write_remainder) = write.as_chunks_mut::<CHUNK_SIZE>();

        for (read_chunk, write_chunk) in read_chunks.iter().zip(write_chunks) {
            Self::prepare_and_send_exactly(lut, src, read_chunk, write_chunk);
        }

        Self::prepare_and_send_remainder(lut, src, read_remainder, write_remainder);
    }
}
