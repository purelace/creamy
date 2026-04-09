cfg_if::cfg_if! {
    if #[cfg(target_feature = "avx512f")] {
        pub const MAX_SLICE_SIZE: usize = 5;
        #[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct Offsets {
            pub avx512f: u8,
            pub avx2: u8,
            pub sse41: u8,
            pub scalar: u8,
            pub ignore: u8,
        }
    } else if #[cfg(target_feature = "avx2")] {
        pub const MAX_SLICE_SIZE: usize = 5;
        #[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct Offsets {
            pub batch: u8,
            pub avx2: u8,
            pub sse41: u8,
            pub scalar: u8,
        }
    } else if #[cfg(target_feature = "sse4.1")] {
        pub const MAX_SLICE_SIZE: usize = 4;
        #[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct Offsets {
            pub batch: u8,
            pub sse41: u8,
            pub scalar: u8,
            pub ignore: u8,
        }
    } else {
        pub const MAX_SLICE_SIZE: usize = 3;
        #[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct Offsets {
            pub batch: u8,
            pub scalar: u8,
            pub ignore: u8,
        }
    }
}

impl Offsets {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
