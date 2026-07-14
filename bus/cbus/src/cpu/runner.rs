use crate::{
    core::UntypedMessage,
    cpu::{PipelineData, set::InstructionSet},
    sys::Header,
};

pub trait InstructionRunner<const CHUNK_SIZE: usize>: InstructionSet<CHUNK_SIZE> {
    #[allow(unused)]
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

    fn prepare_and_send_direct_all(subscribers: &[u8], data: &mut PipelineData) {
        let capacity = data.memory.read.slice_capacity();
        for src in subscribers.iter().copied() {
            let src = src as usize;
            let header_ptr = data.memory.read.header_mut_ptr_for(src);
            let messages = Header::read_slice_mut_test(header_ptr, capacity);
            Self::prepare_and_send_direct_slice(data, src, messages);
            Header::set_count(header_ptr, 0);
        }
    }
}
