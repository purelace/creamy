cfg_select! {
    target_feature = "avx512f" => {
        mod avx512;
        mod avx2;
        mod sse41;

        use avx2::Avx2InstructionSet;
        use avx512::Avx512FInstructionSet;
        use sse41::Sse41InstructionSet;

        use super::generic::ScalarInstructionSet;

        pub type AvailableStrategy = Avx512F;
        define_strategy! {
            Avx512F,
            32.. => [avx512f | Avx512FInstructionSet],
            8..32 => [avx2    | Avx2InstructionSet],
            4..8  => [sse41   | Sse41InstructionSet],
            1..4  => [scalar  | ScalarInstructionSet]
        }
    }
    target_feature = "avx2" => {
        mod avx2;
        mod sse41;

        use avx2::Avx2InstructionSet;
        use sse41::Sse41InstructionSet;

        use super::generic::ScalarInstructionSet;

        pub type AvailableStrategy = Avx2;
        define_strategy! {
            Avx2,
            128..  => [batch  | Avx2InstructionSet],
            8..128 => [avx2   | Avx2InstructionSet],
            4..8   => [sse41    | Sse41InstructionSet],
            1..4   => [scalar | ScalarInstructionSet]
        }
    }
    target_feature = "sse4.1" => {
        mod sse41;

        use sse41::Sse41InstructionSet;

        use super::generic::ScalarInstructionSet;

        pub type AvailableStrategy = Sse41;
        define_strategy! {
            Sse41,
            128..  => [batch  | Sse41InstructionSet],
            4..128 => [sse41    | Sse41InstructionSet],
            1..4   => [scalar | ScalarInstructionSet],
        }
    }
    _ => {
        use super::generic::ScalarInstructionSet;
        define_strategy! {
            Fallback,
            128..  => [batch  | ScalarInstructionSet],
            1..128   => [scalar | ScalarInstructionSet],
        }

        pub type AvailableStrategy = Fallback;
    }
}

#[cfg(target_feature = "sse4.1")]
#[inline(always)]
fn validate_dst(range: crate::bus::SubscriberRange, src: u8, dst: u8) -> u8 {
    let in_valid_range = u8::from(range.contains(dst));
    let not_equal_src = u8::from(src != dst);
    dst * not_equal_src * in_valid_range
}
