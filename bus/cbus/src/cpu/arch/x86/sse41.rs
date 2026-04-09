use core::arch::x86_64::__m128i;
#[cfg(target_feature = "avx2")]
pub use core::arch::x86_64::_mm_blend_epi32;

mod sse41_reexports {
    pub use core::arch::x86_64::_mm_extract_epi32;
}

mod sse2_reexports {
    pub use core::arch::x86_64::{
        _mm_and_si128, _mm_andnot_si128, _mm_cmpeq_epi32, _mm_load_si128, _mm_or_si128,
        _mm_set_epi32, _mm_set1_epi32, _mm_setzero_si128, _mm_shuffle_epi32, _mm_slli_epi32,
        _mm_srli_epi32, _mm_store_si128,
    };
}

use cast_guard::CastGuard;
pub use sse2_reexports::*;
pub use sse41_reexports::*;

use crate::{
    core::UntypedMessage,
    cpu::{
        PipelineData, arch::generic::ScalarInstructionSet, runner::InstructionRunner,
        set::InstructionSet,
    },
    lookup::LookupTable,
};

pub struct Sse41InstructionSet;
impl Sse41InstructionSet {
    // Вносим коррективы в заголовки
    #[inline(always)]
    fn get_valid_header(
        lut: &LookupTable,
        src: usize,
        xmm_msg0: __m128i,
        xmm_msg1: __m128i,
        xmm_msg2: __m128i,
        xmm_msg3: __m128i,
    ) -> __m128i {
        unsafe {
            let bits = lut.max_groups().trailing_zeros() as usize;
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

            // Достаем 'group' (второй байт)
            let xmm_group = _mm_and_si128(_mm_srli_epi32(xmm_raw_headers, 8), _mm_set1_epi32(0xFF));

            macro_rules! translate {
                ($idx:expr) => {{
                    let dst: usize = _mm_extract_epi32(xmm_dst, $idx).safe_cast();
                    let grp: usize = _mm_extract_epi32(xmm_group, $idx).safe_cast();

                    // IN_LUT: [src][local_group]
                    let g_grp = *lut.get_input().get_unchecked(shifted_id + grp);

                    // OUT_LUT: [dst][global_group]
                    let l_grp = *lut
                        .get_output()
                        .get_unchecked((dst << bits) + g_grp as usize);
                    let l_grp: i32 = l_grp.safe_cast();
                    l_grp
                }};
            }

            // Достаем группы
            let l0 = translate!(0);
            let l1 = translate!(1);
            let l2 = translate!(2);
            let l3 = translate!(3);

            let xmm_local_group = _mm_set_epi32(l3, l2, l1, l0);

            // Создаем маску: 0xFFFFFFFF если l_grp != 0, иначе 0
            let xmm_mask = _mm_cmpeq_epi32(xmm_local_group, _mm_setzero_si128());
            let xmm_mask = _mm_andnot_si128(xmm_mask, _mm_set1_epi32(-1)); // Инвертируем маску

            // Собираем новый заголовок: [src][dst][local_group][kind]
            // ВАЖНО: Kind мы берем из оригинального заголовка (байты 24-31)
            let xmm_kind = _mm_and_si128(
                xmm_raw_headers,
                _mm_set1_epi32(0xFF00_0000u32.cast_signed()),
            );
            let xmm_src = _mm_set1_epi32(src.safe_cast());

            let xmm_new_header = _mm_or_si128(
                xmm_dst, // Src
                _mm_or_si128(
                    _mm_slli_epi32(xmm_src, 8), // Dst
                    _mm_or_si128(
                        _mm_slli_epi32(xmm_local_group, 16), // L_Group
                        xmm_kind,                            // Kind
                    ),
                ),
            );

            // Применяем маску валидности (если l_grp был 0, весь заголовок станет 0)
            _mm_and_si128(xmm_new_header, xmm_mask)
        }
    }

    #[inline(always)]
    fn write_messages(destinations: [*mut __m128i; 4], messages: [*const __m128i; 4]) {
        unsafe {
            _mm_store_si128(destinations[0], _mm_load_si128(messages[0]));
            _mm_store_si128(destinations[1], _mm_load_si128(messages[1]));
            _mm_store_si128(destinations[2], _mm_load_si128(messages[2]));
            _mm_store_si128(destinations[3], _mm_load_si128(messages[3]));
        }
    }

    #[inline(always)]
    fn validate_messages(
        lut: &LookupTable,
        src: usize,
        xmm_msg0: __m128i,
        xmm_msg1: __m128i,
        xmm_msg2: __m128i,
        xmm_msg3: __m128i,
    ) -> (__m128i, __m128i, __m128i, __m128i) {
        let xmm_valid_header =
            Self::get_valid_header(lut, src, xmm_msg0, xmm_msg1, xmm_msg2, xmm_msg3);

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
    fn validate_and_write_messages(
        lut: &LookupTable,
        src: usize,
        destination_halves: [*mut __m128i; 4],
        destinations: [*mut __m128i; 4],
        message_halves: [*const __m128i; 4],
        messages: [*const __m128i; 4],
    ) {
        // Так как половинки никак не изменяются и не читаются,
        // Мы сразу загружаем и выгружаем в их точке назначения
        Self::write_messages(destination_halves, message_halves);

        unsafe {
            //Загружаем первую половину сообщений в регистры
            let mut xmm_msg0 = _mm_load_si128(messages[0]);
            let mut xmm_msg1 = _mm_load_si128(messages[1]);
            let mut xmm_msg2 = _mm_load_si128(messages[2]);
            let mut xmm_msg3 = _mm_load_si128(messages[3]);

            (xmm_msg0, xmm_msg1, xmm_msg2, xmm_msg3) =
                Self::validate_messages(lut, src, xmm_msg0, xmm_msg1, xmm_msg2, xmm_msg3);

            _mm_store_si128(destinations[0], xmm_msg0);
            _mm_store_si128(destinations[1], xmm_msg1);
            _mm_store_si128(destinations[2], xmm_msg2);
            _mm_store_si128(destinations[3], xmm_msg3);
        }
    }
}

impl InstructionSet<4> for Sse41InstructionSet {
    #[inline(always)]
    fn send_exactly(read: &[UntypedMessage; 4], write: &mut [UntypedMessage; 4]) {
        unsafe {
            let dst_ptr = write.as_mut_ptr().cast::<__m128i>();
            let chunk_ptr = std::ptr::from_ref(read).cast::<__m128i>();

            let dst0 = dst_ptr;
            let dst1 = dst_ptr.add(2);
            let dst2 = dst_ptr.add(4);
            let dst3 = dst_ptr.add(6);

            let dst0_half = dst_ptr.add(1);
            let dst1_half = dst_ptr.add(3);
            let dst2_half = dst_ptr.add(5);
            let dst3_half = dst_ptr.add(7);

            // SSE обрабатывает всего лишь 128 бит,
            // поэтому нам нужно получить указатели на каждую половину сообщения
            let msg0 = chunk_ptr;
            let msg1 = chunk_ptr.add(2);
            let msg2 = chunk_ptr.add(4);
            let msg3 = chunk_ptr.add(6);

            let msg0_half = chunk_ptr.add(1);
            let msg1_half = chunk_ptr.add(3);
            let msg2_half = chunk_ptr.add(5);
            let msg3_half = chunk_ptr.add(7);

            let destination_halves = [dst0_half, dst1_half, dst2_half, dst3_half];
            let destinations = [dst0, dst1, dst2, dst3];
            let message_halves = [msg0_half, msg1_half, msg2_half, msg3_half];
            let messages = [msg0, msg1, msg2, msg3];

            // Так как половинки никак не изменяются и не читаются,
            // Мы сразу загружаем и выгружаем в их точке назначения
            Self::write_messages(destination_halves, message_halves);
            Self::write_messages(destinations, messages);
        }
    }

    #[inline(always)]
    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        ScalarInstructionSet::send_to(read, write);
    }

    #[inline(always)]
    fn prepare_and_send_exactly(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage; 4],
        write: &mut [UntypedMessage; 4],
    ) {
        unsafe {
            let dst_ptr = write.as_mut_ptr().cast::<__m128i>();
            let chunk_ptr = std::ptr::from_ref(read).cast::<__m128i>();

            let dst0 = dst_ptr;
            let dst1 = dst_ptr.add(2);
            let dst2 = dst_ptr.add(4);
            let dst3 = dst_ptr.add(6);

            let dst0_half = dst_ptr.add(1);
            let dst1_half = dst_ptr.add(3);
            let dst2_half = dst_ptr.add(5);
            let dst3_half = dst_ptr.add(7);

            // SSE обрабатывает всего лишь 128 бит,
            // поэтому нам нужно получить указатели на каждую половину сообщения
            let msg0 = chunk_ptr;
            let msg1 = chunk_ptr.add(2);
            let msg2 = chunk_ptr.add(4);
            let msg3 = chunk_ptr.add(6);

            let msg0_half = chunk_ptr.add(1);
            let msg1_half = chunk_ptr.add(3);
            let msg2_half = chunk_ptr.add(5);
            let msg3_half = chunk_ptr.add(7);

            Self::validate_and_write_messages(
                lut,
                src,
                [dst0_half, dst1_half, dst2_half, dst3_half],
                [dst0, dst1, dst2, dst3],
                [msg0_half, msg1_half, msg2_half, msg3_half],
                [msg0, msg1, msg2, msg3],
            );
        }
    }

    #[inline(always)]
    fn prepare_and_send_remainder(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        ScalarInstructionSet::prepare_and_send_to(lut, src, read, write);
    }
}

impl InstructionRunner<4> for Sse41InstructionSet {
    #[inline(always)]
    fn prepare_and_send_chunk_to_unknown(
        data: &mut PipelineData,
        src: usize,
        chunk: &mut [UntypedMessage; 4],
    ) {
        unsafe {
            let chunk_ptr = std::ptr::from_ref(chunk).cast::<__m128i>();

            // SSE обрабатывает всего лишь 128 бит,
            // поэтому нам нужно получить указатели на каждую половину сообщения
            let msg0 = chunk_ptr;
            let msg1 = chunk_ptr.add(2);
            let msg2 = chunk_ptr.add(4);
            let msg3 = chunk_ptr.add(6);

            let dst0_value = msg0.cast::<u8>().add(1).read();
            let dst1_value = msg1.cast::<u8>().add(1).read();
            let dst2_value = msg2.cast::<u8>().add(1).read();
            let dst3_value = msg3.cast::<u8>().add(1).read();

            let header_0 = data
                .memory
                .write
                .header_ptr_for(dst0_value as usize)
                .as_mut();
            let header_1 = data
                .memory
                .write
                .header_ptr_for(dst1_value as usize)
                .as_mut();
            let header_2 = data
                .memory
                .write
                .header_ptr_for(dst2_value as usize)
                .as_mut();
            let header_3 = data
                .memory
                .write
                .header_ptr_for(dst3_value as usize)
                .as_mut();

            let dst0 = header_0.write_ptr().cast::<__m128i>().as_ptr();
            header_0.count = (header_0.count + 1) * u32::from(dst0_value != 0);

            let dst1 = header_1.write_ptr().cast::<__m128i>().as_ptr();
            header_1.count = (header_1.count + 1) * u32::from(dst1_value != 0);

            let dst2 = header_2.write_ptr().cast::<__m128i>().as_ptr();
            header_2.count = (header_2.count + 1) * u32::from(dst2_value != 0);

            let dst3 = header_3.write_ptr().cast::<__m128i>().as_ptr();
            header_3.count = (header_3.count + 1) * u32::from(dst3_value != 0);

            let dst0_half = dst0.add(1);
            let dst1_half = dst1.add(1);
            let dst2_half = dst2.add(1);
            let dst3_half = dst3.add(1);

            let msg0_half = msg0.add(1);
            let msg1_half = msg1.add(1);
            let msg2_half = msg2.add(1);
            let msg3_half = msg3.add(1);

            let destinations = [dst0, dst1, dst2, dst3];
            let destination_halves = [dst0_half, dst1_half, dst2_half, dst3_half];
            let messages = [msg0, msg1, msg2, msg3];
            let message_halves = [msg0_half, msg1_half, msg2_half, msg3_half];

            Self::validate_and_write_messages(
                data.lookup_table,
                src,
                destination_halves,
                destinations,
                message_halves,
                messages,
            );
        }
    }

    #[inline(always)]
    fn prepare_and_send_direct_slice(
        data: &mut PipelineData,
        src: usize,
        messages: &mut [UntypedMessage],
    ) {
        let mut chunks = messages.chunks_exact_mut(Self::CHUNK_SIZE);
        for chunk in &mut chunks {
            let chunk = unsafe { &mut *chunk.as_mut_ptr().cast::<[UntypedMessage; 4]>() };
            Self::prepare_and_send_chunk_to_unknown(data, src, chunk);
        }

        ScalarInstructionSet::prepare_and_send_direct_slice(data, src, messages);
    }

    #[inline(always)]
    fn prepare_and_send_direct_all(subscribers: &[u8], data: &mut PipelineData) {
        let capacity = data.memory.read.slice_capacity();
        for src in subscribers.iter().copied() {
            let src = src as usize;
            unsafe {
                let header = data.memory.read.header_ptr_for(src).as_mut();
                let messages = header.read_slice_mut(capacity);
                Self::prepare_and_send_direct_slice(data, src, messages);
                header.count = 0;
            }
        }
    }
}
