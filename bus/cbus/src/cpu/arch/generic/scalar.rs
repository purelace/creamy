use cbus_core::Subscriber;

use crate::{
    core::UntypedMessage,
    cpu::{PipelineData, runner::InstructionRunner, set::InstructionSet},
    lookup::LookupTable,
    sys::SubscriberRange,
};

pub struct ScalarInstructionSet;
impl ScalarInstructionSet {
    // Вносим правки в сообщение:
    // * Устанавливаем src, тем самым убираем возможность подмены сообщений
    // * Проверяем права доступа и корректность dst, если он некорректный, то обнуляется
    #[inline(always)]
    fn prepare_single_message<const MG: usize, const MS: usize>(
        lut: &LookupTable<MG, MS>,
        range: SubscriberRange,
        src: u8,
        message: &mut UntypedMessage,
    ) {
        let dst_in_valid_range = range.contains(message.dst);
        let dst_not_equal_src = message.dst != src;

        let src_index = src as usize * MG + message.protocol as usize;
        let consumer_protocol = unsafe { *lut.as_slice().get_unchecked(src_index) };

        let is_valid = u8::from(consumer_protocol != 0 && dst_in_valid_range && dst_not_equal_src);
        let final_dst = is_valid * message.dst;

        // Устанавливаем актуальное значение
        message.src = src;
        message.dst = final_dst;
        message.protocol = consumer_protocol;
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
    pub fn prepare_and_send_to<const MG: usize, const MS: usize>(
        lut: &LookupTable<MG, MS>,
        range: SubscriberRange,
        src: u8,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        let len = core::cmp::min(read.len(), write.len());
        let read = &read[..len];
        let write = &mut write[..len];

        for i in 0..len {
            unsafe {
                *write.get_unchecked_mut(i) = *read.get_unchecked(i);
                Self::prepare_single_message(lut, range, src, write.get_unchecked_mut(i));
            }
        }
    }
}

impl<S, const MM: usize, const MG: usize, const MS: usize> InstructionSet<S, MM, MG, MS, 1>
    for ScalarInstructionSet
where
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
        lut: &LookupTable<MG, MS>,
        range: SubscriberRange,
        src: u8,
        read: &[UntypedMessage; 1],
        write: &mut [UntypedMessage; 1],
    ) {
        Self::prepare_and_send_to(lut, range, src, read, write);
    }

    #[inline(always)]
    fn prepare_and_send_remainder(
        lut: &LookupTable<MG, MS>,
        range: SubscriberRange,
        src: u8,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        Self::prepare_and_send_to(lut, range, src, read, write);
    }
}

impl<S, const MM: usize, const MG: usize, const MS: usize> InstructionRunner<S, MM, MG, MS, 1>
    for ScalarInstructionSet
where
    S: Subscriber,
{
    #[inline(always)]
    fn prepare_and_send_chunk_to_unknown(
        data: &mut PipelineData<S, MM, MG, MS>,
        src: u8,
        chunk: &mut [UntypedMessage; 1],
    ) {
        let message = &mut chunk[0];
        Self::prepare_single_message(data.lookup_table, data.subscriber_range, src, message);
        let mut buffer = data.memory.get_mut_inc_buf(message.dst as usize);

        let write_ptr = buffer.write_slice_mut(1).as_mut_ptr();
        let read_ptr = core::ptr::from_ref(message);

        // SAFETY: TODO
        unsafe {
            core::ptr::copy_nonoverlapping(read_ptr, write_ptr, 1);

            // Инкрементируем count если dst не равен 0
            // Это нужно чтобы все сообщения с dst == 0 отправлялись в мусорку (/dev/null)
            buffer.add_count(u32::from(message.dst != 0));
            //buffer.set_count((buffer.count() + 1) * u32::from(message.dst != 0));
        }
    }

    #[inline(always)]
    fn prepare_and_send_direct_slice(
        data: &mut PipelineData<S, MM, MG, MS>,
        src: u8,
        messages: &mut [UntypedMessage],
    ) {
        for message in messages.as_chunks_mut::<1>().0 {
            Self::prepare_and_send_chunk_to_unknown(data, src, message);
        }
    }
}
