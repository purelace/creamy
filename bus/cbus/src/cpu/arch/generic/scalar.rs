use as_guard::AsGuard;

use crate::{
    core::UntypedMessage,
    cpu::{PipelineData, runner::InstructionRunner, set::InstructionSet},
    lookup::LookupTable,
    sys::Header,
};

pub struct ScalarInstructionSet;
impl ScalarInstructionSet {
    // Вносим правки в сообщение:
    // * Устанавливаем src, тем самым убираем возможность подмены сообщений
    // * Проверяем права доступа и корректность DST, если он некорректный, то он обнуляется
    #[inline(always)]
    fn prepare_single_message(lut: &LookupTable, src: usize, message: &mut UntypedMessage) {
        let max_groups = lut.max_groups();
        // Устанавливаем актуальное значение
        message.src = src.safe_as();

        let local_group_in = (src * max_groups) + message.group as usize;
        let global_group = (message.dst as usize * max_groups)
            + unsafe { *lut.get_input().get_unchecked(local_group_in) as usize };
        let local_group_out = unsafe { *lut.get_output().get_unchecked(global_group) };

        let is_valid = u8::from((global_group != 0) && (local_group_out != 0));
        let final_dst = is_valid * message.dst;

        message.dst = final_dst;
        message.group = local_group_out;
    }

    #[inline(always)]
    pub fn send_to(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        let len = core::cmp::min(read.len(), write.len());
        let read = &read[..len];
        let write = &mut write[..len];

        for i in 0..len {
            unsafe {
                *write.get_unchecked_mut(i) = *read.get_unchecked(i);
            }
        }
    }

    #[inline(always)]
    pub fn prepare_and_send_to(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        let len = core::cmp::min(read.len(), write.len());
        let read = &read[..len];
        let write = &mut write[..len];

        for i in 0..len {
            unsafe {
                *write.get_unchecked_mut(i) = *read.get_unchecked(i);
                Self::prepare_single_message(lut, src, write.get_unchecked_mut(i));
            }
        }
    }
}

impl InstructionSet<1> for ScalarInstructionSet {
    fn send_exactly(read: &[UntypedMessage; 1], write: &mut [UntypedMessage; 1]) {
        Self::send_to(read, write);
    }

    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        Self::send_to(read, write);
    }

    fn prepare_and_send_exactly(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage; 1],
        write: &mut [UntypedMessage; 1],
    ) {
        Self::prepare_and_send_to(lut, src, read, write);
    }

    fn prepare_and_send_remainder(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        Self::prepare_and_send_to(lut, src, read, write);
    }
}

impl InstructionRunner<1> for ScalarInstructionSet {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn prepare_and_send_chunk_to_unknown(
        _data: &mut PipelineData,
        _src: usize,
        _chunk: &mut [UntypedMessage; 1],
    ) {
        unreachable!()
    }

    #[inline(always)]
    fn prepare_and_send_direct_slice(
        data: &mut PipelineData,
        src: usize,
        messages: &mut [UntypedMessage],
    ) {
        for message in messages {
            Self::prepare_single_message(data.lookup_table, src, message);
            let header = data.memory.write.header_mut_ptr_for(message.dst as usize);
            let write_ptr = Header::write_raw_mut_ptr(header);
            let read_ptr = core::ptr::from_ref(message);
            unsafe {
                core::ptr::copy_nonoverlapping(read_ptr, write_ptr, 1);

                // Инкрементируем count если dst не равен 0
                // Это нужно чтобы все сообщения с dst == 0 отправлялись в мусорку (/dev/null)
                (*header).count = ((*header).count + 1) * u32::from(message.dst != 0);
            }
        }
    }
}
