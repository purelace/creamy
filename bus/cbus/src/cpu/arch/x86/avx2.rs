use core::arch::x86_64::__m256i;

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

use crate::{
    core::UntypedMessage,
    cpu::{
        PipelineData, arch::x86::Sse41InstructionSet, runner::InstructionRunner,
        set::InstructionSet,
    },
    lookup::LookupTable,
};

pub const CHUNK_SIZE: usize = 8;

pub struct Avx2InstructionSet;
impl Avx2InstructionSet {
    #[inline(always)]
    fn get_valid_header(
        lut: &LookupTable,
        src: usize,
        ymm_messages: [__m256i; CHUNK_SIZE],
    ) -> __m256i {
        unsafe {
            let bits = lut.max_groups().trailing_zeros() as usize;
            let shifted_src = src << bits;

            let h0 = _mm256_extract_epi32(ymm_messages[0], 0);
            let h1 = _mm256_extract_epi32(ymm_messages[1], 0);
            let h2 = _mm256_extract_epi32(ymm_messages[2], 0);
            let h3 = _mm256_extract_epi32(ymm_messages[3], 0);
            let h4 = _mm256_extract_epi32(ymm_messages[4], 0);
            let h5 = _mm256_extract_epi32(ymm_messages[5], 0);
            let h6 = _mm256_extract_epi32(ymm_messages[6], 0);
            let h7 = _mm256_extract_epi32(ymm_messages[7], 0);

            let ymm_raw_headers = _mm256_set_epi32(h7, h6, h5, h4, h3, h2, h1, h0);

            // Достаем Dst (первый байт каждого u32)
            let ymm_dst = _mm256_and_si256(ymm_raw_headers, _mm256_set1_epi32(0xFF));

            // Достаем Group (второй байт каждого u32)
            let ymm_group = _mm256_and_si256(
                _mm256_srli_epi32(ymm_raw_headers, 8),
                _mm256_set1_epi32(0xFF),
            );

            macro_rules! translate {
                ($idx:expr) => {{
                    let dst: usize = _mm256_extract_epi32(ymm_dst, $idx).safe_as();
                    let grp: usize = _mm256_extract_epi32(ymm_group, $idx).safe_as();

                    // IN_LUT: [src][local_group]
                    let g_grp = *lut.get_input().get_unchecked(shifted_src + grp);

                    // OUT_LUT: [dst][global_group]
                    let l_grp = *lut
                        .get_output()
                        .get_unchecked((dst << bits) + g_grp as usize);
                    let l_grp: i32 = l_grp.safe_as();
                    l_grp
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

            // 5. Сборка нового заголовка [Src][Dst][L_Group][Kind]
            let ymm_kind = _mm256_and_si256(
                ymm_raw_headers,
                _mm256_set1_epi32(0xFF00_0000_u32.cast_signed()),
            );

            let ymm_src = _mm256_set1_epi32(src.safe_as());

            // Собираем всё воедино
            let ymm_new_header = _mm256_or_si256(
                ymm_dst,
                _mm256_or_si256(
                    _mm256_slli_epi32(ymm_src, 8),
                    _mm256_or_si256(_mm256_slli_epi32(ymm_local_group, 16), ymm_kind),
                ),
            );

            // Применяем маску (зануляем инвалидные)
            _mm256_and_si256(ymm_new_header, ymm_mask)
        }
    }

    #[inline(always)]
    fn validate_messages(
        lut: &LookupTable,
        src: usize,
        mut ymm_msgs: [__m256i; CHUNK_SIZE],
    ) -> [__m256i; CHUNK_SIZE] {
        unsafe {
            let ymm_valid_header = Self::get_valid_header(lut, src, ymm_msgs);

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
    fn validate_and_write_messages(
        lut: &LookupTable,
        src: usize,
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
            ymm_messages = Self::validate_messages(lut, src, ymm_messages);

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
    std::array::from_fn(|i| (&raw const from[i]).cast::<__m256i>())
}

#[inline(always)]
fn cast_mut(from: &mut [UntypedMessage; 8]) -> [*mut __m256i; 8] {
    std::array::from_fn(|i| (&raw mut from[i]).cast::<__m256i>())
}

impl InstructionSet<8> for Avx2InstructionSet {
    #[inline(always)]
    fn send_exactly(read: &[UntypedMessage; 8], write: &mut [UntypedMessage; 8]) {
        let messages = cast(read);
        let destinations = cast_mut(write);
        Self::write_messages(&destinations, &messages);
    }

    #[inline(always)]
    fn send_remainder(read: &[UntypedMessage], write: &mut [UntypedMessage]) {
        Sse41InstructionSet::slices_send(read, write);
    }

    #[inline(always)]
    fn prepare_and_send_exactly(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage; 8],
        write: &mut [UntypedMessage; 8],
    ) {
        let messages = cast(read);
        let destinations = cast_mut(write);
        Self::validate_and_write_messages(lut, src, &destinations, &messages);
    }

    #[inline(always)]
    fn prepare_and_send_remainder(
        lut: &LookupTable,
        src: usize,
        read: &[UntypedMessage],
        write: &mut [UntypedMessage],
    ) {
        Sse41InstructionSet::slices_prepare_and_send(lut, src, read, write);
    }
}

impl InstructionRunner<8> for Avx2InstructionSet {
    #[inline(always)]
    fn prepare_and_send_chunk_to_unknown(
        data: &mut PipelineData,
        src: usize,
        chunk: &mut [UntypedMessage; CHUNK_SIZE],
    ) {
        unsafe {
            let chunk_ptr = std::ptr::from_ref(chunk).cast::<UntypedMessage>();

            let msg0 = chunk_ptr.cast::<__m256i>();
            let msg1 = msg0.add(1);
            let msg2 = msg0.add(2);
            let msg3 = msg0.add(3);
            let msg4 = msg0.add(4);
            let msg5 = msg0.add(5);
            let msg6 = msg0.add(6);
            let msg7 = msg0.add(7);

            let dst_0 = msg0.cast::<u8>().add(1).read();
            let dst_1 = msg1.cast::<u8>().add(1).read();
            let dst_2 = msg2.cast::<u8>().add(1).read();
            let dst_3 = msg3.cast::<u8>().add(1).read();
            let dst_4 = msg4.cast::<u8>().add(1).read();
            let dst_5 = msg5.cast::<u8>().add(1).read();
            let dst_6 = msg6.cast::<u8>().add(1).read();
            let dst_7 = msg7.cast::<u8>().add(1).read();

            let header_0 = data.memory.write.header_ptr_for(dst_0 as usize).as_mut();
            let header_1 = data.memory.write.header_ptr_for(dst_1 as usize).as_mut();
            let header_2 = data.memory.write.header_ptr_for(dst_2 as usize).as_mut();
            let header_3 = data.memory.write.header_ptr_for(dst_3 as usize).as_mut();
            let header_4 = data.memory.write.header_ptr_for(dst_4 as usize).as_mut();
            let header_5 = data.memory.write.header_ptr_for(dst_5 as usize).as_mut();
            let header_6 = data.memory.write.header_ptr_for(dst_6 as usize).as_mut();
            let header_7 = data.memory.write.header_ptr_for(dst_7 as usize).as_mut();

            let write_ptr_0 = header_0.write_ptr().cast::<__m256i>().as_ptr();
            header_0.count = (header_0.count + 1) * u32::from(dst_0 != 0);

            let write_ptr_1 = header_1.write_ptr().cast::<__m256i>().as_ptr();
            header_1.count = (header_1.count + 1) * u32::from(dst_1 != 0);

            let write_ptr_2 = header_2.write_ptr().cast::<__m256i>().as_ptr();
            header_2.count = (header_2.count + 1) * u32::from(dst_2 != 0);

            let write_ptr_3 = header_3.write_ptr().cast::<__m256i>().as_ptr();
            header_3.count = (header_3.count + 1) * u32::from(dst_3 != 0);

            let write_ptr_4 = header_4.write_ptr().cast::<__m256i>().as_ptr();
            header_4.count = (header_4.count + 1) * u32::from(dst_4 != 0);

            let write_ptr_5 = header_5.write_ptr().cast::<__m256i>().as_ptr();
            header_5.count = (header_5.count + 1) * u32::from(dst_5 != 0);

            let write_ptr_6 = header_6.write_ptr().cast::<__m256i>().as_ptr();
            header_6.count = (header_6.count + 1) * u32::from(dst_6 != 0);

            let write_ptr_7 = header_7.write_ptr().cast::<__m256i>().as_ptr();
            header_7.count = (header_7.count + 1) * u32::from(dst_7 != 0);

            Self::validate_and_write_messages(
                data.lookup_table,
                src,
                &[
                    write_ptr_0,
                    write_ptr_1,
                    write_ptr_2,
                    write_ptr_3,
                    write_ptr_4,
                    write_ptr_5,
                    write_ptr_6,
                    write_ptr_7,
                ],
                &[msg0, msg1, msg2, msg3, msg4, msg5, msg6, msg7],
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
            Self::prepare_and_send_chunk_to_unknown(data, src, chunk.try_into().unwrap());
        }

        Sse41InstructionSet::prepare_and_send_direct_slice(data, src, messages);
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
