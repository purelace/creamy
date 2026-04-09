use crate::{
    core::UntypedMessage,
    cpu::{PipelineData, set::InstructionSet},
};

pub trait InstructionRunner<const CHUNK_SIZE: usize>: InstructionSet<CHUNK_SIZE> {
    fn prepare_and_send_chunk_to_unknown(
        data: &mut PipelineData,
        src: usize,
        chunk: &mut [UntypedMessage; CHUNK_SIZE],
    );

    fn prepare_and_send_direct_slice(
        data: &mut PipelineData,
        src: usize,
        messages: &mut [UntypedMessage],
    );

    fn prepare_and_send_direct_all(subscribers: &[u8], data: &mut PipelineData);
}
