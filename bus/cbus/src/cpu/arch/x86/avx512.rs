use core::arch::x86_64::{
    __m512i, _kand_mask16, _mm512_add_epi32, _mm512_and_si512, _mm512_cmpneq_epi32_mask,
    _mm512_i32gather_epi32, _mm512_loadu_si512, _mm512_mask_blend_epi32, _mm512_mask_expand_epi32,
    _mm512_maskz_compress_epi32, _mm512_maskz_mov_epi32, _mm512_or_si512, _mm512_permutexvar_epi32,
    _mm512_set_epi32, _mm512_set1_epi32, _mm512_setzero_si512, _mm512_slli_epi32,
    _mm512_sllv_epi32, _mm512_srli_epi32, _mm512_storeu_si512,
};

use as_guard::AsGuard;

use crate::{
    core::UntypedMessage,
    cpu::{
        PipelineData,
        arch::{generic::ScalarInstructionSet, x86::Avx2InstructionSet},
        set::InstructionSet,
    },
    lookup::LookupTable,
    sys::Header,
};

pub struct Avx512FInstructionSet;
impl Avx512FInstructionSet {
    #[inline(always)]
    fn get_valid_header(lut: &LookupTable, src: usize, zmm_messages: &[__m512i; 8]) -> __m512i {
        unsafe {
            let shift_bits = lut.max_groups().trailing_zeros();
            let zmm_bits = _mm512_set1_epi32(shift_bits.safe_as());

            let idx_mask = 0b0000_0001_0000_0001_u16;
            let h0 = _mm512_maskz_compress_epi32(idx_mask, zmm_messages[0]);
            let h1 = _mm512_maskz_compress_epi32(idx_mask, zmm_messages[1]);
            let h2 = _mm512_maskz_compress_epi32(idx_mask, zmm_messages[2]);
            let h3 = _mm512_maskz_compress_epi32(idx_mask, zmm_messages[3]);
            let h4 = _mm512_maskz_compress_epi32(idx_mask, zmm_messages[4]);
            let h5 = _mm512_maskz_compress_epi32(idx_mask, zmm_messages[5]);
            let h6 = _mm512_maskz_compress_epi32(idx_mask, zmm_messages[6]);
            let h7 = _mm512_maskz_compress_epi32(idx_mask, zmm_messages[7]);

            let mut final_headers = _mm512_setzero_si512();
            final_headers = _mm512_mask_expand_epi32(final_headers, 0b0000_0000_0000_0011, h0);
            final_headers = _mm512_mask_expand_epi32(final_headers, 0b0000_0000_0000_1100, h1);
            final_headers = _mm512_mask_expand_epi32(final_headers, 0b0000_0000_0011_0000, h2);
            final_headers = _mm512_mask_expand_epi32(final_headers, 0b0000_0000_1100_0000, h3);
            final_headers = _mm512_mask_expand_epi32(final_headers, 0b0000_0011_0000_0000, h4);
            final_headers = _mm512_mask_expand_epi32(final_headers, 0b0000_1100_0000_0000, h5);
            final_headers = _mm512_mask_expand_epi32(final_headers, 0b0011_0000_0000_0000, h6);
            final_headers = _mm512_mask_expand_epi32(final_headers, 0b1100_0000_0000_0000, h7);

            let zmm_dst = _mm512_and_si512(final_headers, _mm512_set1_epi32(0xFF));
            let zmm_group =
                _mm512_and_si512(_mm512_srli_epi32(final_headers, 8), _mm512_set1_epi32(0xFF));

            let zmm_src = _mm512_set1_epi32(src.safe_as());
            let zmm_local_input_group_indices =
                _mm512_add_epi32(_mm512_sllv_epi32(zmm_src, zmm_bits), zmm_group);

            let zmm_local_input_group_values = _mm512_i32gather_epi32(
                zmm_local_input_group_indices,
                lut.get_input().as_ptr().cast::<i32>(),
                1,
            );

            let zmm_local_input_group_values =
                _mm512_and_si512(zmm_local_input_group_values, _mm512_set1_epi32(0xFF));

            let zmm_global_group = _mm512_add_epi32(
                _mm512_sllv_epi32(zmm_dst, zmm_bits),
                zmm_local_input_group_values,
            );

            let zmm_local_output_group = _mm512_i32gather_epi32(
                zmm_global_group,
                lut.get_output().as_ptr().cast::<i32>(),
                1,
            );
            let zmm_local_output_group =
                _mm512_and_si512(zmm_local_output_group, _mm512_set1_epi32(0xFF));

            let k_global_not_zero =
                _mm512_cmpneq_epi32_mask(zmm_global_group, _mm512_setzero_si512());
            let k_out_not_zero =
                _mm512_cmpneq_epi32_mask(zmm_local_output_group, _mm512_setzero_si512());

            let k_final = _kand_mask16(k_global_not_zero, k_out_not_zero);

            let v_final_dst = _mm512_maskz_mov_epi32(k_final, zmm_dst);
            let v_final_group = zmm_local_output_group;

            let v_combined = _mm512_or_si512(v_final_dst, _mm512_slli_epi32(v_final_group, 8));
            let zmm_header = _mm512_or_si512(v_combined, _mm512_slli_epi32(zmm_src, 16));

            // Вырезаем 4-й байт (маска 0xFF000000)
            // Оставляем только тип сообщения, обнуляя старые dst, group, src
            let zmm_old_type = _mm512_and_si512(
                final_headers,
                _mm512_set1_epi32(0xFF00_0000_u32.cast_signed()),
            );

            // Твой текущий v_header содержит [id][group][dst] и 0 в 4-м байте
            // Склеиваем их: [тип][id][group][dst]
            _mm512_or_si512(zmm_header, zmm_old_type)
        }
    }

    #[inline(always)]
    fn validate_messages(
        lut: &LookupTable,
        src: usize,
        mut zmm_messages: [__m512i; 8],
    ) -> [__m512i; 8] {
        // Маска: меняем только 0-й и 8-й слоты (начало двух сообщений в регистре)
        const MASK: u16 = 0x0101;

        unsafe {
            let zmm_valid_header = Self::get_valid_header(lut, src, &zmm_messages);

            macro_rules! blend_zmm {
                ($msg:expr, $h_idx_low:expr, $h_idx_high:expr) => {{
                    // Создаем индексы для перестановки:
                    // Мы хотим, чтобы в слоте 0 оказался заголовок № $h_idx_low,
                    // а в слоте 8 — заголовок № $h_idx_high. Остальные не важны (их закроет маска).
                    let idx = _mm512_set_epi32(
                        15, 14, 13, 12, 11, 10, 9, $h_idx_high, // Слот 8
                        7, 6, 5, 4, 3, 2, 1, $h_idx_low         // Слот 0
                    );
                    // Вытаскиваем нужные заголовки в позиции 0 и 8
                    let headers = _mm512_permutexvar_epi32(idx, zmm_valid_header);
                    // Вклеиваем их в сообщения
                    $msg = _mm512_mask_blend_epi32(MASK, $msg, headers);
                }};
            }

            // Погнали по парам сообщений (всего 16 сообщений в 8 регистрах)
            blend_zmm!(zmm_messages[0], 0, 1);
            blend_zmm!(zmm_messages[1], 2, 3);
            blend_zmm!(zmm_messages[2], 4, 5);
            blend_zmm!(zmm_messages[3], 6, 7);
            blend_zmm!(zmm_messages[4], 8, 9);
            blend_zmm!(zmm_messages[5], 10, 11);
            blend_zmm!(zmm_messages[6], 12, 13);
            blend_zmm!(zmm_messages[7], 14, 15);
        }

        zmm_messages
    }

    #[inline(always)]
    fn write_messages(destinations: &[*mut __m512i; 8], pairs: &[*const __m512i; 8]) {
        unsafe {
            _mm512_storeu_si512(destinations[0], _mm512_loadu_si512(pairs[0]));
            _mm512_storeu_si512(destinations[1], _mm512_loadu_si512(pairs[1]));
            _mm512_storeu_si512(destinations[2], _mm512_loadu_si512(pairs[2]));
            _mm512_storeu_si512(destinations[3], _mm512_loadu_si512(pairs[3]));
            _mm512_storeu_si512(destinations[4], _mm512_loadu_si512(pairs[4]));
            _mm512_storeu_si512(destinations[5], _mm512_loadu_si512(pairs[5]));
            _mm512_storeu_si512(destinations[6], _mm512_loadu_si512(pairs[6]));
            _mm512_storeu_si512(destinations[7], _mm512_loadu_si512(pairs[7]));
        }
    }

    #[inline(always)]
    fn validate_and_write_messages(
        lut: &LookupTable,
        src: usize,
        destinations: &[*mut __m512i; 8],
        pairs: &[*const __m512i; 8],
    ) {
        unsafe {
            let zmm_msg_pair0 = _mm512_loadu_si512(pairs[0]);
            let zmm_msg_pair1 = _mm512_loadu_si512(pairs[1]);
            let zmm_msg_pair2 = _mm512_loadu_si512(pairs[2]);
            let zmm_msg_pair3 = _mm512_loadu_si512(pairs[3]);
            let zmm_msg_pair4 = _mm512_loadu_si512(pairs[4]);
            let zmm_msg_pair5 = _mm512_loadu_si512(pairs[5]);
            let zmm_msg_pair6 = _mm512_loadu_si512(pairs[6]);
            let zmm_msg_pair7 = _mm512_loadu_si512(pairs[7]);

            let mut zmm_messages = [
                zmm_msg_pair0,
                zmm_msg_pair1,
                zmm_msg_pair2,
                zmm_msg_pair3,
                zmm_msg_pair4,
                zmm_msg_pair5,
                zmm_msg_pair6,
                zmm_msg_pair7,
            ];
            zmm_messages = Self::validate_messages(lut, src, zmm_messages);

            _mm512_storeu_si512(destinations[0], zmm_messages[0]);
            _mm512_storeu_si512(destinations[1], zmm_messages[1]);
            _mm512_storeu_si512(destinations[2], zmm_messages[2]);
            _mm512_storeu_si512(destinations[3], zmm_messages[3]);
            _mm512_storeu_si512(destinations[4], zmm_messages[4]);
            _mm512_storeu_si512(destinations[5], zmm_messages[5]);
            _mm512_storeu_si512(destinations[6], zmm_messages[6]);
            _mm512_storeu_si512(destinations[7], zmm_messages[7]);
        }
    }
}

#[inline(always)]
fn cast(from: &[UntypedMessage; 16]) -> [*const __m512i; 8] {
    // 1. Сразу берем сырой указатель на весь массив
    let base_ptr: *const UntypedMessage = from.as_ptr();

    // 2. Строим массив указателей, отталкиваясь ТОЛЬКО от базового сырого указателя
    core::array::from_fn(|i| unsafe { base_ptr.add(i * 2).cast::<__m512i>() })
}

#[inline(always)]
fn cast_mut(from: &mut [UntypedMessage; 16]) -> [*mut __m512i; 8] {
    core::array::from_fn(|i| (&raw mut from[i * 2]).cast::<__m512i>())
}

impl InstructionSet<16> for Avx512FInstructionSet {
    #[inline(always)]
    fn send_exactly(read: &[UntypedMessage; 16], write: &mut [UntypedMessage; 16]) {
        let pairs = cast(read);
        let destinations = cast_mut(write);
        Self::write_messages(&destinations, &pairs);
    }

    #[inline(always)]
    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        Avx2InstructionSet::slices_send(read, write);
    }

    #[inline(always)]
    fn prepare_and_send_exactly(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage; 16],
        write: &mut [UntypedMessage; 16],
    ) {
        let pairs = cast(read);
        let destinations = cast_mut(write);
        Self::validate_and_write_messages(lut, src, &destinations, &pairs);
    }

    #[inline(always)]
    fn prepare_and_send_remainder(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        Avx2InstructionSet::slices_prepare_and_send(lut, src, read, write);
    }

    #[inline(always)]
    fn prepare_batches(subscribers: &[u8], data: &mut PipelineData) {
        let capacity = data.memory.read.slice_capacity();
        for src in subscribers.iter().copied() {
            let src = src as usize;
            let header = data.memory.read.header_mut_ptr_for(src);
            let read = Header::read_slice_mut_test(header, capacity);

            unsafe {
                let write = data.memory.message.reserve_slice((*header).count as usize);

                // Обрабатываем нечетное сообщение.
                let is_odd = usize::from(!read.len().is_multiple_of(2));
                let (read_odd, read) = read.split_at(is_odd);
                let (write_odd, write) = write.split_at_mut(is_odd);
                ScalarInstructionSet::prepare_and_send_to(
                    data.lookup_table,
                    src,
                    read_odd,
                    write_odd,
                );

                Self::slices_prepare_and_send(data.lookup_table, src, read, write);

                Header::set_count(header, 0);
            }
        }
    }
}
