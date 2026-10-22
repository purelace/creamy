use core::arch::x86_64::__m128i;
#[cfg(target_feature = "avx2")]
pub use core::arch::x86_64::_mm_blend_epi32;
#[cfg(not(target_feature = "avx2"))]
pub use core::arch::x86_64::{_mm_blend_ps, _mm_castps_si128, _mm_castsi128_ps};

mod sse41_reexports {
    pub use core::arch::x86_64::_mm_extract_epi32;
}

mod sse2_reexports {
    pub use core::arch::x86_64::{
        _mm_and_si128, _mm_andnot_si128, _mm_cmpeq_epi32, _mm_load_si128, _mm_max_epi32,
        _mm_min_epi32, _mm_or_si128, _mm_set_epi32, _mm_set1_epi32, _mm_setzero_si128,
        _mm_shuffle_epi32, _mm_slli_epi32, _mm_srli_epi32, _mm_store_si128,
    };
}

use as_guard::AsGuard;
use cbus_core::Subscriber;
pub use sse2_reexports::*;
pub use sse41_reexports::*;

use super::validate_dst;
use crate::{
    bus::SubscriberRange,
    config::BusConfig,
    core::UntypedMessage,
    cpu::{
        PipelineData, arch::generic::ScalarInstructionSet, runner::InstructionRunner,
        set::InstructionSet,
    },
    lookup::LookupTable,
};

pub struct Sse41InstructionSet;
impl Sse41InstructionSet {
    // Проверяем, находится ли dst в допустимом диапазоне
    // Validate dst (1)
    #[inline(always)]
    fn validate_dst(range: SubscriberRange, xmm_dst: __m128i) -> __m128i {
        unsafe {
            let min = _mm_set1_epi32(i32::from(range.start().cast_signed()));
            let max = _mm_set1_epi32(i32::from(range.end().cast_signed()));

            let xmm_is_dst_in_valid_range =
                _mm_cmpeq_epi32(_mm_min_epi32(_mm_max_epi32(xmm_dst, min), max), xmm_dst);
            _mm_and_si128(xmm_dst, xmm_is_dst_in_valid_range)
        }
    }

    // Вносим коррективы в заголовки
    #[inline(always)]
    fn get_valid_header<C: BusConfig>(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: usize,
        xmm_msg0: __m128i,
        xmm_msg1: __m128i,
        xmm_msg2: __m128i,
        xmm_msg3: __m128i,
    ) -> __m128i {
        unsafe {
            let bits = LookupTable::<C>::MAX_GROUPS.trailing_zeros() as usize;
            let shifted_id = src << bits;

            // Берем первые 4 байта [dst][group][src][kind]
            let h0 = _mm_extract_epi32(xmm_msg0, 0);
            let h1 = _mm_extract_epi32(xmm_msg1, 0);
            let h2 = _mm_extract_epi32(xmm_msg2, 0);
            let h3 = _mm_extract_epi32(xmm_msg3, 0);

            // Загружаем 4 заголовка в одно значение
            let xmm_raw_headers = _mm_set_epi32(h3, h2, h1, h0);

            // Достаем 'dst' (первый байт)
            let xmm_dst = _mm_and_si128(xmm_raw_headers, _mm_set1_epi32(0xFF));
            let xmm_dst = Self::validate_dst(range, xmm_dst);

            // Достаем 'group' (второй байт)
            let xmm_group = _mm_and_si128(_mm_srli_epi32(xmm_raw_headers, 8), _mm_set1_epi32(0xFF));

            macro_rules! translate {
                ($idx:expr) => {{
                    let group: usize = _mm_extract_epi32(xmm_group, $idx).safe_as();

                    (*lut.get_input().get_unchecked(shifted_id + group)).safe_as()
                }};
            }

            // Достаем группы
            let l0 = translate!(0);
            let l1 = translate!(1);
            let l2 = translate!(2);
            let l3 = translate!(3);

            let xmm_group = _mm_set_epi32(l3, l2, l1, l0);

            // Создаем маску: 0xFFFFFFFF если group != 0, иначе 0
            let xmm_mask = _mm_cmpeq_epi32(xmm_group, _mm_setzero_si128());
            let xmm_mask = _mm_andnot_si128(xmm_mask, _mm_set1_epi32(-1)); // Инвертируем маску

            // Собираем новый заголовок: [dst][group][src][kind]
            let xmm_kind = _mm_and_si128(
                xmm_raw_headers,
                _mm_set1_epi32(0xFF00_0000u32.cast_signed()),
            );

            let xmm_src = _mm_set1_epi32(src.safe_as());

            let xmm_new_header = _mm_or_si128(
                xmm_dst,
                _mm_or_si128(
                    _mm_slli_epi32(xmm_group, 8),
                    _mm_or_si128(_mm_slli_epi32(xmm_src, 16), xmm_kind),
                ),
            );

            // Применяем маску валидности (если group был 0, весь заголовок станет 0)
            _mm_and_si128(xmm_new_header, xmm_mask)
        }
    }

    #[inline(always)]
    fn validate_messages<C: BusConfig>(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: usize,
        xmm_msg0: __m128i,
        xmm_msg1: __m128i,
        xmm_msg2: __m128i,
        xmm_msg3: __m128i,
    ) -> (__m128i, __m128i, __m128i, __m128i) {
        let xmm_valid_header =
            Self::get_valid_header(lut, range, src, xmm_msg0, xmm_msg1, xmm_msg2, xmm_msg3);

        unsafe {
            // H0 уже в начале [H3, H2, H1, H0]
            // Сдвигаем H1 в начало: [--, H3, H2, H1]
            // Сдвигаем H2 в начало: [--, --, H3, H2]
            // Сдвигаем H3 в начало: [--, --, --, H3]
            let h0 = xmm_valid_header;
            let h1 = _mm_shuffle_epi32::<0b01_01_01_01>(xmm_valid_header);
            let h2 = _mm_shuffle_epi32::<0b10_10_10_10>(xmm_valid_header);
            let h3 = _mm_shuffle_epi32::<0b11_11_11_11>(xmm_valid_header);

            // We have two variants how to rewrite the message header.
            // AVX2 extension [_mm_blend_epi32]
            // SSE4.1 + SSE2 [_mm_blend_ps, _mm_castps_si128, _mm_castsi128_ps]
            #[cfg(target_feature = "avx2")]
            {
                let m0 = _mm_blend_epi32(xmm_msg0, h0, 0b0001);
                let m1 = _mm_blend_epi32(xmm_msg1, h1, 0b0001);
                let m2 = _mm_blend_epi32(xmm_msg2, h2, 0b0001);
                let m3 = _mm_blend_epi32(xmm_msg3, h3, 0b0001);
                (m0, m1, m2, m3)
            }

            #[cfg(not(target_feature = "avx2"))]
            {
                let m0 = _mm_castps_si128(_mm_blend_ps::<0b0001>(
                    _mm_castsi128_ps(xmm_msg0),
                    _mm_castsi128_ps(h0),
                ));
                let m1 = _mm_castps_si128(_mm_blend_ps::<0b0001>(
                    _mm_castsi128_ps(xmm_msg1),
                    _mm_castsi128_ps(h1),
                ));
                let m2 = _mm_castps_si128(_mm_blend_ps::<0b0001>(
                    _mm_castsi128_ps(xmm_msg2),
                    _mm_castsi128_ps(h2),
                ));
                let m3 = _mm_castps_si128(_mm_blend_ps::<0b0001>(
                    _mm_castsi128_ps(xmm_msg3),
                    _mm_castsi128_ps(h3),
                ));
                (m0, m1, m2, m3)
            }
        }
    }

    #[inline(always)]
    fn validate_and_write_messages<C: BusConfig>(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: usize,
        messages: &XmmMessages,
    ) {
        // Так как половинки никак не изменяются и не читаются,
        // Мы сразу загружаем и выгружаем в их точке назначения
        messages.write_part2();

        unsafe {
            //Загружаем первую половину сообщений в регистры
            let mut xmm_msg0 = _mm_load_si128(messages.messages_part1[0]);
            let mut xmm_msg1 = _mm_load_si128(messages.messages_part1[1]);
            let mut xmm_msg2 = _mm_load_si128(messages.messages_part1[2]);
            let mut xmm_msg3 = _mm_load_si128(messages.messages_part1[3]);

            (xmm_msg0, xmm_msg1, xmm_msg2, xmm_msg3) =
                Self::validate_messages(lut, range, src, xmm_msg0, xmm_msg1, xmm_msg2, xmm_msg3);

            _mm_store_si128(messages.dst_part1[0], xmm_msg0);
            _mm_store_si128(messages.dst_part1[1], xmm_msg1);
            _mm_store_si128(messages.dst_part1[2], xmm_msg2);
            _mm_store_si128(messages.dst_part1[3], xmm_msg3);
        }
    }
}

/// * Part1: \[Header 4 Bytes\] \[Payload 12 Bytes\]
/// * Part2: \[Payload 16 Bytes\]
struct XmmMessages {
    messages_part1: [*const __m128i; 4],
    messages_part2: [*const __m128i; 4],
    dst_part1: [*mut __m128i; 4],
    dst_part2: [*mut __m128i; 4],
}

impl XmmMessages {
    #[inline(always)]
    const fn new(read: &[UntypedMessage; 4], write: &mut [UntypedMessage; 4]) -> Self {
        unsafe {
            // SSE обрабатывает всего лишь 128 бит,
            // поэтому нам нужно получить указатели на каждую половину сообщения
            let chunk_ptr = core::ptr::from_ref(read).cast::<__m128i>();
            let msg0 = chunk_ptr;
            let msg1 = chunk_ptr.add(2);
            let msg2 = chunk_ptr.add(4);
            let msg3 = chunk_ptr.add(6);

            let msg0_half = chunk_ptr.add(1);
            let msg1_half = chunk_ptr.add(3);
            let msg2_half = chunk_ptr.add(5);
            let msg3_half = chunk_ptr.add(7);

            // SSE обрабатывает всего лишь 128 бит,
            // поэтому нам нужно получить указатели на каждую половину сообщения
            let dst_ptr = write.as_mut_ptr().cast::<__m128i>();
            let dst0 = dst_ptr;
            let dst1 = dst_ptr.add(2);
            let dst2 = dst_ptr.add(4);
            let dst3 = dst_ptr.add(6);

            let dst0_half = dst_ptr.add(1);
            let dst1_half = dst_ptr.add(3);
            let dst2_half = dst_ptr.add(5);
            let dst3_half = dst_ptr.add(7);

            let dst_part1 = [dst0, dst1, dst2, dst3];
            let dst_part2 = [dst0_half, dst1_half, dst2_half, dst3_half];

            Self {
                messages_part1: [msg0, msg1, msg2, msg3],
                messages_part2: [msg0_half, msg1_half, msg2_half, msg3_half],
                dst_part1,
                dst_part2,
            }
        }
    }

    #[inline(always)]
    fn write_part1(&self) {
        unsafe {
            _mm_store_si128(self.dst_part1[0], _mm_load_si128(self.messages_part1[0]));
            _mm_store_si128(self.dst_part1[1], _mm_load_si128(self.messages_part1[1]));
            _mm_store_si128(self.dst_part1[2], _mm_load_si128(self.messages_part1[2]));
            _mm_store_si128(self.dst_part1[3], _mm_load_si128(self.messages_part1[3]));
        }
    }

    #[inline(always)]
    fn write_part2(&self) {
        unsafe {
            _mm_store_si128(self.dst_part2[0], _mm_load_si128(self.messages_part2[0]));
            _mm_store_si128(self.dst_part2[1], _mm_load_si128(self.messages_part2[1]));
            _mm_store_si128(self.dst_part2[2], _mm_load_si128(self.messages_part2[2]));
            _mm_store_si128(self.dst_part2[3], _mm_load_si128(self.messages_part2[3]));
        }
    }
}

impl<C, S, const M: usize> InstructionSet<C, S, M, 4> for Sse41InstructionSet
where
    C: BusConfig,
    S: Subscriber,
{
    #[inline(always)]
    fn send_exactly(read: &[UntypedMessage; 4], write: &mut [UntypedMessage; 4]) {
        // Так как половинки никак не изменяются и не читаются,
        // Мы сразу загружаем и выгружаем в их точке назначения
        let messages = XmmMessages::new(read, write);
        messages.write_part1();
        messages.write_part2();
    }

    #[inline(always)]
    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        ScalarInstructionSet::send_to(read, write);
    }

    #[inline(always)]
    fn prepare_and_send_exactly(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: usize,
        read: &[UntypedMessage; 4],
        write: &mut [UntypedMessage; 4],
    ) {
        let messages = XmmMessages::new(read, write);
        Self::validate_and_write_messages(lut, range, src, &messages);
    }

    #[inline(always)]
    fn prepare_and_send_remainder(
        lut: &LookupTable<C>,
        range: SubscriberRange,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        ScalarInstructionSet::prepare_and_send_to(lut, range, src, read, write);
    }
}

impl<C, S, const M: usize> InstructionRunner<C, S, M, 4> for Sse41InstructionSet
where
    C: BusConfig,
    S: Subscriber,
{
    #[inline(always)]
    fn prepare_and_send_chunk_to_unknown(
        data: &mut PipelineData<C, S, M>,
        src: usize,
        chunk: &mut [UntypedMessage; 4],
    ) {
        unsafe {
            let chunk_ptr = core::ptr::from_ref(chunk).cast::<__m128i>();

            // SSE обрабатывает всего лишь 128 бит,
            // поэтому нам нужно получить указатели на каждую половину сообщения
            let msg0 = chunk_ptr;
            let msg1 = chunk_ptr.add(2);
            let msg2 = chunk_ptr.add(4);
            let msg3 = chunk_ptr.add(6);

            let dst0 = msg0.cast::<u8>().add(1).read();
            let dst1 = msg1.cast::<u8>().add(1).read();
            let dst2 = msg2.cast::<u8>().add(1).read();
            let dst3 = msg3.cast::<u8>().add(1).read();

            // Validate dst (0)
            let dst0 = validate_dst(data.subscriber_range, dst0);
            let dst1 = validate_dst(data.subscriber_range, dst1);
            let dst2 = validate_dst(data.subscriber_range, dst2);
            let dst3 = validate_dst(data.subscriber_range, dst3);

            let mut h0 = data.memory.get_inc_raw_buf(dst0);
            let mut h1 = data.memory.get_inc_raw_buf(dst1);
            let mut h2 = data.memory.get_inc_raw_buf(dst2);
            let mut h3 = data.memory.get_inc_raw_buf(dst3);

            let dst0_ptr = h0.write_raw_mut_ptr().cast::<__m128i>();
            let dst1_ptr = h1.write_raw_mut_ptr().cast::<__m128i>();
            let dst2_ptr = h2.write_raw_mut_ptr().cast::<__m128i>();
            let dst3_ptr = h3.write_raw_mut_ptr().cast::<__m128i>();

            let dst0_ptr_half = dst0_ptr.add(1);
            let dst1_ptr_half = dst1_ptr.add(1);
            let dst2_ptr_half = dst2_ptr.add(1);
            let dst3_ptr_half = dst3_ptr.add(1);

            let msg0_half = msg0.add(1);
            let msg1_half = msg1.add(1);
            let msg2_half = msg2.add(1);
            let msg3_half = msg3.add(1);

            let dst_part1 = [dst0_ptr, dst1_ptr, dst2_ptr, dst3_ptr];
            let dst_part2 = [dst0_ptr_half, dst1_ptr_half, dst2_ptr_half, dst3_ptr_half];
            let messages_part1 = [msg0, msg1, msg2, msg3];
            let messages_part2 = [msg0_half, msg1_half, msg2_half, msg3_half];

            let messages = XmmMessages {
                messages_part1,
                messages_part2,
                dst_part1,
                dst_part2,
            };

            Self::validate_and_write_messages(
                data.lookup_table,
                data.subscriber_range,
                src,
                &messages,
            );

            h0.set_count((h0.count() + 1) * u32::from(dst0 != 0));
            h1.set_count((h1.count() + 1) * u32::from(dst1 != 0));
            h2.set_count((h2.count() + 1) * u32::from(dst2 != 0));
            h3.set_count((h3.count() + 1) * u32::from(dst3 != 0));
        }
    }

    #[inline(always)]
    fn prepare_and_send_direct_slice(
        data: &mut PipelineData<C, S, M>,
        src: usize,
        messages: &mut [UntypedMessage],
    ) {
        //TODO: remainder

        let mut chunks =
            messages.chunks_exact_mut(<Self as InstructionSet<C, S, M, _>>::CHUNK_SIZE);
        for chunk in &mut chunks {
            let chunk = unsafe { &mut *chunk.as_mut_ptr().cast::<[UntypedMessage; 4]>() };
            Self::prepare_and_send_chunk_to_unknown(data, src, chunk);
        }

        ScalarInstructionSet::prepare_and_send_direct_slice(data, src, messages);
    }
}
