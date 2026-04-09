use core::arch::x86_64::{
    __m512i, _kand_mask16, _mm512_add_epi32, _mm512_and_si512, _mm512_cmpneq_epi32_mask,
    _mm512_i32gather_epi32, _mm512_mask_blend_epi32, _mm512_mask_expand_epi32,
    _mm512_maskz_compress_epi32, _mm512_maskz_mov_epi32, _mm512_or_si512, _mm512_permutexvar_epi32,
    _mm512_set_epi32, _mm512_set1_epi32, _mm512_setzero_si512, _mm512_slli_epi32,
    _mm512_sllv_epi32, _mm512_srli_epi32,
};
use std::arch::x86_64::{_mm512_loadu_si512, _mm512_storeu_si512};

use cast_guard::CastGuard;

use crate::{
    core::UntypedMessage,
    cpu::{
        MessagePipeline, PipelineData,
        arch::{generic::ScalarInstructionSet, x86::Avx2InstructionSet},
        set::InstructionSet,
    },
    lookup::LookupTable,
};

impl MessagePipeline {
    /*
       fn avx512_prepare_and_send_to_unknown(
           &self,
           data: &mut PipelineData,
           src: usize,
           read: &mut [UntypedMessage; CHUNK_SIZE],
       ) {
           unsafe {
               let chunk_ptr = std::ptr::from_ref(read).cast::<UntypedMessage>();

               macro_rules! write_ptr {
                   ($index:expr) => {{
                       let dst = read[$index].dst as usize;
                       let header = data.write_pool.header_ptr_for(dst).as_mut();
                       let write_ptr = header.write_ptr().cast::<__m512i>().as_ptr();
                       header.count = (header.count + 1) * u32::from(dst != 0);
                       write_ptr
                   }};
               }

               let msg0_1 = chunk_ptr.cast::<__m512i>();
               let msg2_3 = msg0_1.add(1);
               let msg4_5 = msg2_3.add(1);
               let msg6_7 = msg4_5.add(1);
               let msg8_9 = msg6_7.add(1);
               let msg10_11 = msg8_9.add(1);
               let msg12_13 = msg10_11.add(1);
               let msg14_15 = msg12_13.add(1);

               let mut zmm_msg0_1 = _mm512_load_si512(msg0_1);
               let mut zmm_msg2_3 = _mm512_load_si512(msg2_3);
               let mut zmm_msg4_5 = _mm512_load_si512(msg4_5);
               let mut zmm_msg6_7 = _mm512_load_si512(msg6_7);
               let mut zmm_msg8_9 = _mm512_load_si512(msg8_9);
               let mut zmm_msg10_11 = _mm512_load_si512(msg10_11);
               let mut zmm_msg12_13 = _mm512_load_si512(msg12_13);
               let mut zmm_msg14_15 = _mm512_load_si512(msg14_15);

               let zmm_valid_header = self.get_valid_header_zmm(
                   src,
                   zmm_msg0_1,
                   zmm_msg2_3,
                   zmm_msg4_5,
                   zmm_msg6_7,
                   zmm_msg8_9,
                   zmm_msg10_11,
                   zmm_msg12_13,
                   zmm_msg14_15,
               );

               // Маска: меняем только 0-й и 8-й слоты (начало двух сообщений в ZMM)
               const MASK: u16 = 0x0101;

               macro_rules! blend_zmm {
                   ($msg:ident, $h_idx_low:expr, $h_idx_high:expr) => {{
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
               blend_zmm!(zmm_msg0_1, 0, 1);
               blend_zmm!(zmm_msg2_3, 2, 3);
               blend_zmm!(zmm_msg4_5, 4, 5);
               blend_zmm!(zmm_msg6_7, 6, 7);
               blend_zmm!(zmm_msg8_9, 8, 9);
               blend_zmm!(zmm_msg10_11, 10, 11);
               blend_zmm!(zmm_msg12_13, 12, 13);
               blend_zmm!(zmm_msg14_15, 14, 15);
           }
       }
    */
    /*
    pub(crate) fn avx512_send(&mut self) {
        unsafe {
            self.batch
                .iter()
                .for_each(|&(destination, to_write, ptr_location)| {
                    let header = self.write_pool.header_for(destination as usize);
                    let mut dst_ptr = header.write_ptr().cast::<__m512i>().as_ptr();
                    let mut src_ptr = self.pool.ptr_at(ptr_location).cast::<__m512i>();

                    // Сразу записываем итоговое значение если dst != 0
                    // Это нужно чтобы все сообщения с dst == 0 отправлялись в мусорку (/dev/null)
                    {
                        let ptr = src_ptr.cast::<UntypedMessage>().as_ref_unchecked();
                        header.count = (header.count + to_write) * u32::from(ptr.dst != 0);
                    }

                    // Так как мы используем AVX512, то делим количество итераций на два.
                    let to_write = to_write.midpoint(to_write % 2);
                    for _ in 0..to_write {
                        let m = _mm512_loadu_si512(src_ptr);
                        _mm512_store_si512(dst_ptr, m);

                        // Перемещаем указатель на два слота вперед (два сообщения вперед)
                        src_ptr = src_ptr.add(1);
                        dst_ptr = dst_ptr.add(1);
                    }
                });
        }

        self.batch.clear();
        self.pool.clear();
    }

    pub(crate) fn avx512_prepare(&mut self) {
        let capacity = self.read_pool.slice_capacity();

        for dst in 1..=self.mint.last() as usize {
            unsafe {
                let header = self.read_pool.header_for(dst);
                align_debug_assert!(header.data.as_ptr(), TARGET_ALIGN);

                let actual_count = header.count as usize;
                let midpoint_count = actual_count.midpoint(actual_count % 2);

                header.count = 0;

                let mut read_ptr = header.read_ptr(capacity).as_ptr().cast::<__m512i>();

                self.driver.on_message_prepare_batch(
                    dst,
                    std::slice::from_raw_parts_mut(read_ptr.cast::<UntypedMessage>(), actual_count),
                );

                // Заранее резервируем память для сообщений, чтобы не тратить время на инкрементацию счетчика
                let mut dst_ptr = self.pool.reserve(actual_count).cast::<__m512i>();

                for _ in 0..midpoint_count {
                    align_debug_assert!(read_ptr.cast::<u8>(), TARGET_ALIGN / 2);
                    align_debug_assert!(dst_ptr.cast::<u8>(), TARGET_ALIGN);

                    // Используем инструкцию для невыровненых адресов,
                    // так как данных в буфере может быть не четное количество
                    let m = _mm512_loadu_si512(read_ptr);
                    _mm512_store_si512(dst_ptr, m);

                    // Перемещаем указатели на два слота вперед (два сообщения вперед)
                    read_ptr = read_ptr.add(1);
                    dst_ptr = dst_ptr.add(1);
                }
            }
        }

        self.pool.as_mut_slice().sort_unstable_by_key(|m| m.dst);

        let mut ptr_location = 0;
        let pool_slice = self.pool.as_slice();

        // Находим индекс первого элемента, где dst != 0
        let start_index = pool_slice.partition_point(|x| x.dst == 0);
        let active_slice = &pool_slice[start_index..];

        // Обновляем начальный ptr_location, чтобы он соответствовал пропущенным данным
        ptr_location += start_index;

        active_slice
            .chunk_by(|a, b| a.dst == b.dst)
            .for_each(|slice| {
                let dst = slice[0].dst; // slice гарантированно не пустой
                self.batch
                    .push((dst, slice.len().safe_cast(), ptr_location));
                ptr_location += slice.len();
            });
    }
    */
}

pub struct Avx512FInstructionSet;
impl Avx512FInstructionSet {
    /*
    #[inline(always)]
    fn get_valid_header(lut: &LookupTable, src: usize, zmm_messages: &[__m512i; 8]) -> __m512i {
        unsafe {
            let shift_bits = lut.max_groups().trailing_zeros();
            let zmm_bits = _mm512_set1_epi32(shift_bits.safe_cast());

            // Индексы для сборки: берем 0-й и 8-й элементы из каждой пары ZMM
            // 0, 8 (из первого), 16, 24 (из второго) и так далее...
            let idx_low = _mm512_set_epi32(24, 16, 8, 0, 24, 16, 8, 0, 24, 16, 8, 0, 24, 16, 8, 0);

            // Склеиваем пары регистров (0+1, 2+3, 4+5, 6+7)
            // Это заменяет 8 вызовов compress
            let p01 = _mm512_permutex2var_epi32(zmm_messages[0], idx_low, zmm_messages[1]);
            let p23 = _mm512_permutex2var_epi32(zmm_messages[2], idx_low, zmm_messages[3]);
            let p45 = _mm512_permutex2var_epi32(zmm_messages[4], idx_low, zmm_messages[5]);
            let p67 = _mm512_permutex2var_epi32(zmm_messages[6], idx_low, zmm_messages[7]);

            // Финальная сборка всех заголовков в один ZMM
            // Это заменяет 8 вызовов expand
            let idx_final_01 =
                _mm512_set_epi32(31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 3, 2, 1, 0);
            let h_mid_1 = _mm512_permutex2var_epi32(p01, idx_final_01, p23);
            let h_mid_2 = _mm512_permutex2var_epi32(p45, idx_final_01, p67);

            let final_headers = _mm512_permutex2var_epi32(h_mid_1, idx_final_01, h_mid_2);

            // --- Дальше твоя логика обработки ---

            let zmm_dst = _mm512_and_si512(final_headers, _mm512_set1_epi32(0xFF));
            let zmm_group =
                _mm512_and_si512(_mm512_srli_epi32(final_headers, 8), _mm512_set1_epi32(0xFF));

            let zmm_src = _mm512_set1_epi32(src.safe_cast());
            let zmm_local_input_group_indices =
                _mm512_add_epi32(_mm512_sllv_epi32(zmm_src, zmm_bits), zmm_group);

            // Gather 1: Input Group
            let mut zmm_input_vals = _mm512_i32gather_epi32(
                zmm_local_input_group_indices,
                lut.get_input().as_ptr().cast::<i32>(),
                1,
            );
            zmm_input_vals = _mm512_and_si512(zmm_input_vals, _mm512_set1_epi32(0xFF));

            let zmm_global_group =
                _mm512_add_epi32(_mm512_sllv_epi32(zmm_dst, zmm_bits), zmm_input_vals);

            // Gather 2: Output Group
            let mut zmm_out_group = _mm512_i32gather_epi32(
                zmm_global_group,
                lut.get_output().as_ptr().cast::<i32>(),
                1,
            );
            zmm_out_group = _mm512_and_si512(zmm_out_group, _mm512_set1_epi32(0xFF));

            // Маскирование и финальная сборка через Ternary Logic (vpternlogd)
            let k_final = _kand_mask16(
                _mm512_cmpneq_epi32_mask(zmm_global_group, _mm512_setzero_si512()),
                _mm512_cmpneq_epi32_mask(zmm_out_group, _mm512_setzero_si512()),
            );

            let v_final_dst = _mm512_maskz_mov_epi32(k_final, zmm_dst);

            // Сборка финального заголовка: комбинируем dst, group, src и старый тип сообщения
            // Используем логическое ИЛИ для сборки полей
            let v_combined = _mm512_or_si512(v_final_dst, _mm512_slli_epi32(zmm_out_group, 8));
            let zmm_new_fields = _mm512_or_si512(v_combined, _mm512_slli_epi32(zmm_src, 16));

            // Оставляем только 4-й байт (тип) из оригинальных заголовков
            let zmm_old_type = _mm512_and_si512(
                final_headers,
                _mm512_set1_epi32(0xFF00_0000_u32.cast_signed()),
            );

            _mm512_or_si512(zmm_new_fields, zmm_old_type)
        }
    } */

    #[inline(always)]
    fn get_valid_header(lut: &LookupTable, src: usize, zmm_messages: &[__m512i; 8]) -> __m512i {
        unsafe {
            let shift_bits = lut.max_groups().trailing_zeros();
            let zmm_bits = _mm512_set1_epi32(shift_bits.safe_cast());

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

            let zmm_src = _mm512_set1_epi32(src.safe_cast());
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
            //TODO alignment
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
    std::array::from_fn(|i| (&raw const from[i * 2]).cast::<__m512i>())
}

#[inline(always)]
fn cast_mut(from: &mut [UntypedMessage; 16]) -> [*mut __m512i; 8] {
    std::array::from_fn(|i| (&raw mut from[i * 2]).cast::<__m512i>())
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
        /* [] [][][][][][] */

        let capacity = data.memory.read.slice_capacity();
        for src in subscribers.iter().copied() {
            let src = src as usize;
            let header = data.memory.read.header_for(src);

            let read = header.read_slice(capacity);
            let write = data.memory.message.reserve_slice(header.count as usize);

            // Обрабатываем нечетное сообщение.
            let is_odd = usize::from(!read.len().is_multiple_of(2));
            let (read_odd, read) = read.split_at(is_odd);
            let (write_odd, write) = write.split_at_mut(is_odd);
            ScalarInstructionSet::prepare_and_send_to(data.lookup_table, src, read_odd, write_odd);

            Self::slices_prepare_and_send(data.lookup_table, src, read, write);

            header.count = 0;
        }
    }
}
