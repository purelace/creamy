use core::arch::x86_64::{__m256i, _mm256_max_epi32, _mm256_min_epi32};

mod avx_reexport {
    pub use core::arch::x86_64::{
        _mm256_extract_epi32, _mm256_load_si256, _mm256_set_epi32, _mm256_set1_epi32,
        _mm256_setzero_si256, _mm256_store_si256,
    };
}

mod avx2_reexport {
    pub use core::arch::x86_64::{
        _mm256_and_si256, _mm256_andnot_si256, _mm256_blend_epi32, _mm256_cmpeq_epi32,
        _mm256_or_si256, _mm256_permutevar8x32_epi32, _mm256_slli_epi32, _mm256_srli_epi32,
    };
}

use as_guard::AsGuard;
pub use avx_reexport::*;
pub use avx2_reexport::*;
use cbus_core::Subscriber;

use super::validate_dst;
use crate::{
    config::BusConfig,
    core::UntypedMessage,
    cpu::{
        PipelineData, arch::x86::Sse41InstructionSet, runner::InstructionRunner,
        set::InstructionSet,
    },
    lookup::LookupTable,
    sys::SubscriberRange,
};

pub const CHUNK_SIZE: usize = 8;

pub struct Avx2InstructionSet;
impl Avx2InstructionSet {
    #[inline(always)]
    fn validate_dst(range: SubscriberRange, src: u8, ymm_dst: __m256i) -> __m256i {
        unsafe {
            let min = _mm256_set1_epi32(i32::from(range.start().cast_signed()));
            let max = _mm256_set1_epi32(i32::from(range.end().cast_signed()));

            let ymm_is_dst_in_valid_range = _mm256_cmpeq_epi32(
                _mm256_min_epi32(_mm256_max_epi32(ymm_dst, min), max),
                ymm_dst,
            );
            let ymm_dst = _mm256_and_si256(ymm_dst, ymm_is_dst_in_valid_range);

            let src = i32::from(src.cast_signed());
            let ymm_src = _mm256_set1_epi32(src);

            let ymm_mask = _mm256_cmpeq_epi32(ymm_dst, ymm_src);
            let all_ones = _mm256_cmpeq_epi32(ymm_mask, ymm_mask);
            let ymm_dst_not_equal_src = _mm256_andnot_si256(ymm_mask, all_ones);

            _mm256_and_si256(ymm_dst, ymm_dst_not_equal_src)
        }
    }

    #[inline(always)]
    fn get_valid_header<C: BusConfig>(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: u8,
        ymm_messages: [__m256i; CHUNK_SIZE],
    ) -> __m256i {
        unsafe {
            let bits = LookupTable::<C>::MAX_GROUPS.trailing_zeros() as usize;
            let shifted_src = (src as usize) << bits;

            let h0 = _mm256_extract_epi32(ymm_messages[0], 0);
            let h1 = _mm256_extract_epi32(ymm_messages[1], 0);
            let h2 = _mm256_extract_epi32(ymm_messages[2], 0);
            let h3 = _mm256_extract_epi32(ymm_messages[3], 0);
            let h4 = _mm256_extract_epi32(ymm_messages[4], 0);
            let h5 = _mm256_extract_epi32(ymm_messages[5], 0);
            let h6 = _mm256_extract_epi32(ymm_messages[6], 0);
            let h7 = _mm256_extract_epi32(ymm_messages[7], 0);

            let ymm_raw_headers = _mm256_set_epi32(h7, h6, h5, h4, h3, h2, h1, h0);

            // Достаем dst [0]
            let ymm_dst = _mm256_and_si256(ymm_raw_headers, _mm256_set1_epi32(0xFF));
            let ymm_dst = Self::validate_dst(range, src, ymm_dst);

            // Достаем group [1]
            let ymm_group = _mm256_and_si256(
                _mm256_srli_epi32(ymm_raw_headers, 8),
                _mm256_set1_epi32(0xFF),
            );

            macro_rules! translate {
                ($idx:expr) => {{
                    let group: usize = _mm256_extract_epi32(ymm_group, $idx).safe_as();
                    (*lut.as_slice().get_unchecked(shifted_src + group)).safe_as()
                }};
            }

            let l0 = translate!(0);
            let l1 = translate!(1);
            let l2 = translate!(2);
            let l3 = translate!(3);
            let l4 = translate!(4);
            let l5 = translate!(5);
            let l6 = translate!(6);
            let l7 = translate!(7);

            let ymm_local_group = _mm256_set_epi32(l7, l6, l5, l4, l3, l2, l1, l0);

            let ymm_zero = _mm256_setzero_si256();
            let ymm_mask = _mm256_cmpeq_epi32(ymm_local_group, ymm_zero);
            let ymm_mask = _mm256_andnot_si256(ymm_mask, _mm256_set1_epi32(-1));

            // 5. Сборка нового заголовка [dst][group][src][kind]
            let ymm_kind = _mm256_and_si256(
                ymm_raw_headers,
                _mm256_set1_epi32(0xFF00_0000_u32.cast_signed()),
            );

            let ymm_src = _mm256_set1_epi32(src.safe_as());

            // Собираем всё воедино
            let ymm_new_header = _mm256_or_si256(
                ymm_dst,
                _mm256_or_si256(
                    _mm256_slli_epi32(ymm_local_group, 8),
                    _mm256_or_si256(_mm256_slli_epi32(ymm_src, 16), ymm_kind),
                ),
            );

            // Применяем маску (зануляем инвалидные)
            _mm256_and_si256(ymm_new_header, ymm_mask)
        }
    }

    #[inline(always)]
    fn validate_messages<C: BusConfig>(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: u8,
        mut ymm_msgs: [__m256i; CHUNK_SIZE],
    ) -> [__m256i; CHUNK_SIZE] {
        unsafe {
            let ymm_valid_header = Self::get_valid_header(lut, range, src, ymm_msgs);

            macro_rules! get_valid {
                ($idx:expr, $ymm_msg:expr) => {{
                    let ymm_idx = _mm256_set1_epi32($idx);
                    let ymm_header = _mm256_permutevar8x32_epi32(ymm_valid_header, ymm_idx);
                    _mm256_blend_epi32($ymm_msg, ymm_header, 0b0001)
                }};
            }

            ymm_msgs[0] = get_valid!(0, ymm_msgs[0]);
            ymm_msgs[1] = get_valid!(1, ymm_msgs[1]);
            ymm_msgs[2] = get_valid!(2, ymm_msgs[2]);
            ymm_msgs[3] = get_valid!(3, ymm_msgs[3]);
            ymm_msgs[4] = get_valid!(4, ymm_msgs[4]);
            ymm_msgs[5] = get_valid!(5, ymm_msgs[5]);
            ymm_msgs[6] = get_valid!(6, ymm_msgs[6]);
            ymm_msgs[7] = get_valid!(7, ymm_msgs[7]);

            ymm_msgs
        }
    }

    #[inline(always)]
    fn write_messages(destinations: &[*mut __m256i; 8], messages: &[*const __m256i; 8]) {
        unsafe {
            _mm256_store_si256(destinations[0], _mm256_load_si256(messages[0]));
            _mm256_store_si256(destinations[1], _mm256_load_si256(messages[1]));
            _mm256_store_si256(destinations[2], _mm256_load_si256(messages[2]));
            _mm256_store_si256(destinations[3], _mm256_load_si256(messages[3]));
            _mm256_store_si256(destinations[4], _mm256_load_si256(messages[4]));
            _mm256_store_si256(destinations[5], _mm256_load_si256(messages[5]));
            _mm256_store_si256(destinations[6], _mm256_load_si256(messages[6]));
            _mm256_store_si256(destinations[7], _mm256_load_si256(messages[7]));
        }
    }

    #[inline(always)]
    fn validate_and_write_messages<C: BusConfig>(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: u8,
        destinations: &[*mut __m256i; 8],
        messages: &[*const __m256i; 8],
    ) {
        unsafe {
            let ymm_msg0 = _mm256_load_si256(messages[0]);
            let ymm_msg1 = _mm256_load_si256(messages[1]);
            let ymm_msg2 = _mm256_load_si256(messages[2]);
            let ymm_msg3 = _mm256_load_si256(messages[3]);
            let ymm_msg4 = _mm256_load_si256(messages[4]);
            let ymm_msg5 = _mm256_load_si256(messages[5]);
            let ymm_msg6 = _mm256_load_si256(messages[6]);
            let ymm_msg7 = _mm256_load_si256(messages[7]);

            let mut ymm_messages = [
                ymm_msg0, ymm_msg1, ymm_msg2, ymm_msg3, ymm_msg4, ymm_msg5, ymm_msg6, ymm_msg7,
            ];
            ymm_messages = Self::validate_messages(lut, range, src, ymm_messages);

            _mm256_store_si256(destinations[0], ymm_messages[0]);
            _mm256_store_si256(destinations[1], ymm_messages[1]);
            _mm256_store_si256(destinations[2], ymm_messages[2]);
            _mm256_store_si256(destinations[3], ymm_messages[3]);
            _mm256_store_si256(destinations[4], ymm_messages[4]);
            _mm256_store_si256(destinations[5], ymm_messages[5]);
            _mm256_store_si256(destinations[6], ymm_messages[6]);
            _mm256_store_si256(destinations[7], ymm_messages[7]);
        }
    }
}

#[inline(always)]
fn cast(from: &[UntypedMessage; 8]) -> [*const __m256i; 8] {
    core::array::from_fn(|i| (&raw const from[i]).cast::<__m256i>())
}

#[inline(always)]
fn cast_mut(from: &mut [UntypedMessage; 8]) -> [*mut __m256i; 8] {
    core::array::from_fn(|i| (&raw mut from[i]).cast::<__m256i>())
}

impl<C, S, const M: usize> InstructionSet<C, S, M, 8> for Avx2InstructionSet
where
    C: BusConfig,
    S: Subscriber,
{
    #[inline(always)]
    fn send_exactly(read: &[UntypedMessage; 8], write: &mut [UntypedMessage; 8]) {
        let messages = cast(read);
        let destinations = cast_mut(write);
        Self::write_messages(&destinations, &messages);
    }

    #[inline(always)]
    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        <Sse41InstructionSet as InstructionSet<C, S, M, _>>::slices_send(read, write);
    }

    #[inline(always)]
    fn prepare_and_send_exactly(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: u8,
        read: &[UntypedMessage; 8],
        write: &mut [UntypedMessage; 8],
    ) {
        let messages = cast(read);
        let destinations = cast_mut(write);
        Self::validate_and_write_messages(lut, range, src, &destinations, &messages);
    }

    #[inline(always)]
    fn prepare_and_send_remainder(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: u8,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        <Sse41InstructionSet as InstructionSet<C, S, M, _>>::slices_prepare_and_send(
            lut, range, src, read, write,
        );
    }
}

impl<C, S, const M: usize> InstructionRunner<C, S, M, 8> for Avx2InstructionSet
where
    C: BusConfig,
    S: Subscriber,
{
    #[inline(always)]
    #[allow(clippy::too_many_lines)]
    fn prepare_and_send_chunk_to_unknown(
        data: &mut PipelineData<C, S, M>,
        src: u8,
        chunk: &mut [UntypedMessage; CHUNK_SIZE],
    ) {
        unsafe {
            let chunk_ptr = core::ptr::from_ref(chunk).cast::<UntypedMessage>();

            let msg0 = chunk_ptr.cast::<__m256i>();
            let msg1 = msg0.add(1);
            let msg2 = msg0.add(2);
            let msg3 = msg0.add(3);
            let msg4 = msg0.add(4);
            let msg5 = msg0.add(5);
            let msg6 = msg0.add(6);
            let msg7 = msg0.add(7);

            let dst0 = msg0.cast::<u8>().read();
            let dst1 = msg1.cast::<u8>().read();
            let dst2 = msg2.cast::<u8>().read();
            let dst3 = msg3.cast::<u8>().read();
            let dst4 = msg4.cast::<u8>().read();
            let dst5 = msg5.cast::<u8>().read();
            let dst6 = msg6.cast::<u8>().read();
            let dst7 = msg7.cast::<u8>().read();

            let dst0 = validate_dst(data.subscriber_range, src, dst0);
            let dst1 = validate_dst(data.subscriber_range, src, dst1);
            let dst2 = validate_dst(data.subscriber_range, src, dst2);
            let dst3 = validate_dst(data.subscriber_range, src, dst3);
            let dst4 = validate_dst(data.subscriber_range, src, dst4);
            let dst5 = validate_dst(data.subscriber_range, src, dst5);
            let dst6 = validate_dst(data.subscriber_range, src, dst6);
            let dst7 = validate_dst(data.subscriber_range, src, dst7);

            let mut h0 = data.memory.get_inc_raw_buf(dst0);
            let mut h1 = data.memory.get_inc_raw_buf(dst1);
            let mut h2 = data.memory.get_inc_raw_buf(dst2);
            let mut h3 = data.memory.get_inc_raw_buf(dst3);
            let mut h4 = data.memory.get_inc_raw_buf(dst4);
            let mut h5 = data.memory.get_inc_raw_buf(dst5);
            let mut h6 = data.memory.get_inc_raw_buf(dst6);
            let mut h7 = data.memory.get_inc_raw_buf(dst7);

            let dst0_ptr = h0.write_raw_mut_ptr().cast::<__m256i>();
            h0.set_count((h0.count() + 1) * u32::from(dst0 != 0));
            let dst1_ptr = h1.write_raw_mut_ptr().cast::<__m256i>();
            h1.set_count((h1.count() + 1) * u32::from(dst1 != 0));
            let dst2_ptr = h2.write_raw_mut_ptr().cast::<__m256i>();
            h2.set_count((h2.count() + 1) * u32::from(dst2 != 0));
            let dst3_ptr = h3.write_raw_mut_ptr().cast::<__m256i>();
            h3.set_count((h3.count() + 1) * u32::from(dst3 != 0));
            let dst4_ptr = h4.write_raw_mut_ptr().cast::<__m256i>();
            h4.set_count((h4.count() + 1) * u32::from(dst4 != 0));
            let dst5_ptr = h5.write_raw_mut_ptr().cast::<__m256i>();
            h5.set_count((h5.count() + 1) * u32::from(dst5 != 0));
            let dst6_ptr = h6.write_raw_mut_ptr().cast::<__m256i>();
            h6.set_count((h6.count() + 1) * u32::from(dst6 != 0));
            let dst7_ptr = h7.write_raw_mut_ptr().cast::<__m256i>();
            h7.set_count((h7.count() + 1) * u32::from(dst7 != 0));

            Self::validate_and_write_messages(
                data.lookup_table,
                data.subscriber_range,
                src,
                &[
                    dst0_ptr, dst1_ptr, dst2_ptr, dst3_ptr, dst4_ptr, dst5_ptr, dst6_ptr, dst7_ptr,
                ],
                &[msg0, msg1, msg2, msg3, msg4, msg5, msg6, msg7],
            );
        }
    }

    #[inline(always)]
    fn prepare_and_send_direct_slice(
        data: &mut PipelineData<C, S, M>,
        src: u8,
        messages: &mut [UntypedMessage],
    ) {
        let (chunks, remainder) = messages.as_chunks_mut::<CHUNK_SIZE>();
        for chunk in chunks {
            Self::prepare_and_send_chunk_to_unknown(data, src, chunk);
        }

        Sse41InstructionSet::prepare_and_send_direct_slice(data, src, remainder);
    }
}
