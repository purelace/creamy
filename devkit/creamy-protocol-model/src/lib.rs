#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod definition;
pub mod error;
pub mod ranges;
pub mod storage;
pub mod symbols;
mod table;
mod utils;
mod version;

pub use semver::Version;
pub use table::{FinishedTypeTable, TypeId, TypeMeta, TypeTable};
pub use utils::*;

pub mod constraints {
    // Глобальные ограничения:
    //TODO: move to pool crate
    pub const MAX_UNIQUE_STRINGS: usize = 65536;

    /// * dst: u8,
    /// * group: u8,
    /// * src: u8,
    /// * kind: u8,
    pub const HEADER_BYTES: u8 = 4;

    pub const MAX_GROUPS: usize = 255;
    pub const MAX_MESSAGES_PER_GROUP: usize = 255;

    pub const MAX_STRUCTS: usize = 2048;

    pub const MAX_FLAGS: usize = 2048;
    pub const MAX_OPTIONS: usize = 65536;

    pub const MAX_BITSETS: usize = 2048;
    pub const MAX_BITSET_VALUES: usize = 65536;
    pub const MAX_BITSET_SIZE: usize = MAX_PAYLOAD * 8;

    pub const MAX_ENUMS: usize = 2048;
    pub const MAX_VARIANTS: usize = 65536; //REMOVE

    pub const MAX_FIELDS: usize = 2048;
    pub const MAX_FIELD_PER_STRUCT: usize = 28;

    pub const MAX_PAYLOAD: usize = 28;
    pub const MAX_MESSAGES: usize = MAX_GROUPS * MAX_MESSAGES_PER_GROUP;
    pub const MAX_TYPE_COUNT: usize = MAX_STRUCTS + MAX_ENUMS + MAX_BITSETS + MAX_FLAGS;
}
