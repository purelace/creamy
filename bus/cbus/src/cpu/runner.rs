use cbus_core::Subscriber;

use crate::{
    config::BusConfig,
    core::UntypedMessage,
    cpu::{PipelineData, set::InstructionSet},
};

pub trait InstructionRunner<C, S, const M: usize, const CHUNK_SIZE: usize>:
    InstructionSet<C, S, M, CHUNK_SIZE>
where
    C: BusConfig,
    S: Subscriber,
{
    #[allow(unused)]
    fn prepare_and_send_chunk_to_unknown(
        data: &mut PipelineData<C, S, M>,
        src: usize,
        chunk: &mut [UntypedMessage; CHUNK_SIZE],
    );

    fn prepare_and_send_direct_slice(
        data: &mut PipelineData<C, S, M>,
        src: usize,
        messages: &mut [UntypedMessage],
    );

    #[tracing::instrument(skip_all)]
    fn prepare_and_send_direct_all(subscribers: &[u8], data: &mut PipelineData<C, S, M>) {
        for src in subscribers.iter().copied() {
            let src = src as usize;
            let mut buffer = data.memory.get_mut_out_buf(src);
            let messages = buffer.read_slice_mut();

            //let header_ptr = data.memory.read.header_mut_ptr_for(src);
            //let messages = Header::read_slice_mut_test(header_ptr, capacity);
            Self::prepare_and_send_direct_slice(data, src, messages);

            buffer.set_count(0);
        }
    }
}
