use alloc::string::{String, ToString};

use thiserror::Error;

use crate::symbols::VariantValue;

pub trait Fallback {
    fn fallback() -> Self;
}

const ERROR_IDENT: &str = "Error";

impl Fallback for String {
    fn fallback() -> Self {
        ERROR_IDENT.to_string()
    }
}

impl Fallback for &str {
    fn fallback() -> Self {
        ERROR_IDENT
    }
}

impl Fallback for usize {
    fn fallback() -> Self {
        0
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(miette::Diagnostic))]
#[cfg_attr(feature = "std", diagnostic(severity(Error)))]
pub enum SemanticError {
    // --- Global Limits ---
    #[error("[P101] Too many unique strings. Max 65535")]
    TooManyUniqueStrings,

    #[error("[P200] Invalid size {actual}")]
    #[cfg_attr(feature = "std", diagnostic(code(semantic::invalid_size)))]
    #[cfg_attr(
        feature = "std",
        diagnostic(help("actual size must be between 1 and 28 bytes"))
    )]
    InvalidSize {
        //#[label("Here")]
        //span: SourceSpan,
        //#[related]
        //model: TypeModel,
        actual: usize,
    },

    #[error("[P201] Target align '{actual}' is not power of 2")]
    #[cfg_attr(feature = "std", diagnostic(code(semantic::align_is_not_power_of_two)))]
    AlignIsNotPowerOfTwo { actual: u8 },

    #[error("[P202] Target align cannot be '{actual}'. Probably, this is a bug.")]
    #[cfg_attr(feature = "std", diagnostic(code(semantic::forbidden_align)))]
    #[cfg_attr(feature = "std", diagnostic(help("allowed align: [1, 2, 4, 8, 16]")))]
    ForbiddenAlign { actual: u8 },

    #[error("[P203] Target raw align cannot be '{actual}'. Probably, this is a bug.")]
    #[cfg_attr(feature = "std", diagnostic(code(semantic::forbidden_raw_align)))]
    #[cfg_attr(feature = "std", diagnostic(help("allowed align: [0, 1, 2, 3, 4]")))]
    ForbiddenRawAlign { actual: u8 },

    #[error("[P204] Invalid enum underlying type")]
    #[cfg_attr(
        feature = "std",
        diagnostic(code(semantic::invalid_enum_underlying_type))
    )]
    #[cfg_attr(
        feature = "std",
        diagnostic(help("supported types: u8, u16, u32, u64, i8, i16, i32, i64"))
    )]
    InvalidEnumUnderlyingType,

    #[error("[P205] Too many fields in '{0}'")]
    #[cfg_attr(feature = "std", diagnostic(code(semantic::field_limit_exceeded)))]
    #[cfg_attr(
        feature = "std",
        diagnostic(help("max 28 fields per struct/message is allowed"))
    )]
    FieldLimitExceeded(String),

    #[error("[P206] Not enough free space in '{0}' struct. Max 28 reserved bytes")]
    FreeBytesLimitExceeded(String),

    #[error("[P207] Enum variant value out of range: {value}")]
    #[cfg_attr(
        feature = "std",
        diagnostic(code(semantic::enum_variant_value_out_of_range))
    )]
    #[cfg_attr(feature = "std", diagnostic(help("allowed range ({min}..={max})")))]
    EnumVariantValueOutOfRange {
        value: VariantValue,
        min: i64,
        max: u64,
    },

    #[error("[P208] Cannot resolve type {from}: required type '{kind}' not found.")]
    CannotResolveTypeFieldNotFound { from: String, kind: String },

    #[error("[P209] {0}: Self reference is not allowed.")]
    SelfReference(String),

    #[error("[P210] {0}: Message reference is not allowed.")]
    MessageReference(String),
}
