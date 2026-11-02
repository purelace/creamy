use cbus_core::Subscriber;

use crate::{
    core::UntypedMessage,
    cpu::{PipelineData, set::InstructionSet},
};

pub trait InstructionRunner<
    S,
    const MM: usize,
    const MG: usize,
    const MS: usize,
    const CHUNK_SIZE: usize,
>: InstructionSet<S, MM, MG, MS, CHUNK_SIZE> where
    S: Subscriber,
{
    fn prepare_and_send_chunk_to_unknown(
        data: &mut PipelineData<S, MM, MG, MS>,
        src: u8,
        chunk: &mut [UntypedMessage; CHUNK_SIZE],
    );

    fn prepare_and_send_direct_slice(
        data: &mut PipelineData<S, MM, MG, MS>,
        src: u8,
        messages: &mut [UntypedMessage],
    );

    fn prepare_and_send_direct_all(subscribers: &[u8], data: &mut PipelineData<S, MM, MG, MS>) {
        for src in subscribers.iter().copied() {
            let mut buffer = data.memory.get_mut_out_buf(src);
            let messages = buffer.read_slice_mut();

            Self::prepare_and_send_direct_slice(data, src, messages);

            buffer.clear();
        }
    }
}
