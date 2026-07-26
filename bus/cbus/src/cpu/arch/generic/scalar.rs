use as_guard::AsGuard;
use cbus_core::Subscriber;

use crate::{
    config::BusConfig,
    core::UntypedMessage,
    cpu::{PipelineData, runner::InstructionRunner, set::InstructionSet},
    lookup::LookupTable,
};

pub struct ScalarInstructionSet;
impl ScalarInstructionSet {
    // Вносим правки в сообщение:
    // * Устанавливаем src, тем самым убираем возможность подмены сообщений
    // * Проверяем права доступа и корректность DST, если он некорректный, то он обнуляется
    #[inline(always)]
    fn prepare_single_message<C: BusConfig>(
        lut: &LookupTable<C>,
        src: usize,
        message: &mut UntypedMessage,
    ) {
        // Устанавливаем актуальное значение
        message.src = src.safe_as();

        let in_idx = src * LookupTable::<C>::MAX_GROUPS + message.group as usize;
        let relative_to_dst_group = unsafe { *lut.get_input().get_unchecked(in_idx) };

        let is_valid = u8::from(relative_to_dst_group != 0);
        let final_dst = is_valid * message.dst;

        message.dst = final_dst;
        message.group = relative_to_dst_group;
    }

    #[inline(always)]
    pub fn send_to(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        tracing::info!("got message from");
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
    pub fn prepare_and_send_to<C: BusConfig>(
        lut: &LookupTable<C>,
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

impl<C, S, const M: usize> InstructionSet<C, S, M, 1> for ScalarInstructionSet
where
    C: BusConfig,
    S: Subscriber,
{
    #[inline(always)]
    fn send_exactly(read: &[UntypedMessage; 1], write: &mut [UntypedMessage; 1]) {
        Self::send_to(read, write);
    }

    #[inline(always)]
    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        Self::send_to(read, write);
    }

    #[inline(always)]
    fn prepare_and_send_exactly(
        lut: &LookupTable<C>,
        src: usize,
        read: &[UntypedMessage; 1],
        write: &mut [UntypedMessage; 1],
    ) {
        tracing::info!("got message from: {src}");
        Self::prepare_and_send_to(lut, src, read, write);
    }

    #[inline(always)]
    fn prepare_and_send_remainder(
        lut: &LookupTable<C>,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        tracing::info!("got message from: {src}");
        Self::prepare_and_send_to(lut, src, read, write);
    }
}

impl<C, S, const M: usize> InstructionRunner<C, S, M, 1> for ScalarInstructionSet
where
    C: BusConfig,
    S: Subscriber,
{
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn prepare_and_send_chunk_to_unknown(
        _data: &mut PipelineData<C, S, M>,
        _src: usize,
        _chunk: &mut [UntypedMessage; 1],
    ) {
        unreachable!()
    }

    #[tracing::instrument(skip_all)]
    #[inline(always)]
    fn prepare_and_send_direct_slice(
        data: &mut PipelineData<C, S, M>,
        src: usize,
        messages: &mut [UntypedMessage],
    ) {
        for message in messages {
            Self::prepare_single_message(data.lookup_table, src, message);
            let mut buffer = data.memory.get_mut_inc_buf(message.dst as usize);

            let write_ptr = buffer.write_slice_mut(1).as_mut_ptr();
            let read_ptr = core::ptr::from_ref(message);

            unsafe {
                core::ptr::copy_nonoverlapping(read_ptr, write_ptr, 1);

                // Инкрементируем count если dst не равен 0
                // Это нужно чтобы все сообщения с dst == 0 отправлялись в мусорку (/dev/null)
                buffer.set_count((buffer.count() + 1) * u32::from(message.dst != 0));
            }
        }
    }
}
