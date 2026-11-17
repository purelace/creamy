use std::num::ParseIntError;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::Diagnostics;

#[derive(Debug, Error, Diagnostic, Clone, PartialEq, Eq)]
#[diagnostic(severity(Error))]
pub enum SyntaxError {
    #[error("[P003] Invalid version format")]
    #[diagnostic(code(syntax::invalid_version_format))]
    #[diagnostic(help("allowed syntax: MAJOR.MINOR.PATCH"))]
    InvalidVersionFormat {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P006] Invalid access value")]
    #[diagnostic(code(syntax::invalid_access))]
    #[diagnostic(help("allowed values: Public, Private"))]
    InvalidAccess {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P007] Invalid direction value")]
    #[diagnostic(code(syntax::invalid_direction))]
    #[diagnostic(help("allowed values: Incoming, Outgoing, Duplex"))]
    InvalidDirection {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P009] Int parse error: {error}")]
    #[diagnostic(code(syntax::int_error))]
    IntParse {
        #[label("Here")]
        span: SourceSpan,
        error: ParseIntError,
    },

    #[error("[P012] Not a number")]
    #[diagnostic(code(syntax::nan))]
    NotANumber {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("Syntax Error: expected '{expected}'")]
    UnexpectedToken {
        expected: &'static str,
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("Syntax Error: expected an item")]
    UnexpectedEOF,

    #[error("Syntax Error: unknown keyword '{actual}'")]
    UnknownKeyword {
        actual: String,

        #[label("Here")]
        span: SourceSpan,
    },
}

#[derive(Debug, Error, Diagnostic, Clone, PartialEq, Eq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub enum ProtocolError {
    SyntaxError(#[from] SyntaxError),
    SemanticError(#[from] SemanticError),
}

pub trait ProtocolErrorExt<T: Fallback> {
    fn or_recover(self, diagnostics: &mut Diagnostics) -> T;
}

impl<T: Fallback, E: Into<ProtocolError>> ProtocolErrorExt<T> for Result<T, E> {
    fn or_recover(self, diagnostics: &mut Diagnostics) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                diagnostics.report_err(error);
                T::fallback()
            }
        }
    }
}

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

#[derive(Debug, Error, Clone, PartialEq, Eq, Diagnostic)]
#[diagnostic(severity(Error))]
pub enum SemanticError {
    // --- Global Limits ---
    #[error("[P101] Too many unique strings. Max 65535")]
    TooManyUniqueStrings,

    #[error("[P200] Invalid size {actual}")]
    #[diagnostic(code(semantic::invalid_size))]
    #[diagnostic(help("actual size must be between 1 and 28 bytes"))]
    // TODO: support other types
    // TODO: spans
    InvalidSize {
        //#[label("Here")]
        //span: SourceSpan,
        //#[related]
        //model: TypeModel,
        actual: u64,
    },

    #[error("[P201] Target align '{actual}' is not power of 2")]
    #[diagnostic(code(semantic::align_is_not_power_of_two))]
    AlignIsNotPowerOfTwo { actual: u64 },

    #[error("[P202] Target align cannot be '{actual}'. Probably, this is a bug.")]
    #[diagnostic(code(semantic::forbidden_align))]
    #[diagnostic(help("allowed align: [1, 2, 4, 8, 16]"))]
    ForbiddenAlign { actual: u64 },

    #[error("[P203] Target raw align cannot be '{actual}'. Probably, this is a bug.")]
    #[diagnostic(code(semantic::forbidden_raw_align))]
    #[diagnostic(help("allowed align: [0, 1, 2, 3, 4]"))]
    ForbiddenRawAlign { actual: u8 },

    #[error("[P204] Invalid enum underlying type")]
    #[diagnostic(code(semantic::invalid_enum_underlying_type))]
    #[diagnostic(help("supported types: u8, u16, u32, u64, i8, i16, i32, i64"))]
    InvalidEnumUnderlyingType,

    #[error("[P205] Too many fields in '{0}'")]
    #[diagnostic(code(semantic::field_limit_exceeded))]
    #[diagnostic(help("max 28 fields per struct/message is allowed"))]
    FieldLimitExceeded(String),

    #[error("[P206] Not enough free space in '{0}' struct. Max 28 reserved bytes")]
    FreeBytesLimitExceeded(String),

    #[error("[P207] Enum variant value out of range: {value}")]
    #[diagnostic(code(semantic::enum_variant_value_out_of_range))]
    #[diagnostic(help("allowed range ({min}..={max})"))]
    EnumVariantValueOutOfRange { value: String, min: i64, max: u64 },

    #[error("[P208] Cannot resolve type {from}: required type '{kind}' not found.")]
    CannotResolveTypeFieldNotFound { from: String, kind: String },

    #[error("[P209] {0}: Self reference is not allowed.")]
    SelfReference(String),

    #[error("[P210] {0}: Message reference is not allowed.")]
    MessageReference(String),

    // --- Protocol Limits ---
    #[error("[P211] Too many messages in protocol")]
    #[diagnostic(code(ast::too_many_messages))]
    #[diagnostic(help("max 256"))]
    TooManyMessages,

    #[error(
        "[P212] Forbidden type in bitset value representation: only primitive numeric types are allowed."
    )]
    #[diagnostic(code(semantic::forbidden_type))]
    #[diagnostic(help("u8, u16, u32, u64, u128, i8, i16, i32, i64, i128"))]
    ForbiddenType {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P213] Zero width of bitset values is not allowed")]
    #[diagnostic(code(semantic::zero_value_width))]
    ZeroBitsetValueWidth {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[] Bitset value width {actual} is larger than {repr} (max {max_possible} bits)")]
    BitsetValueExceedsRepr {
        actual: u64,
        repr: String,
        max_possible: u64,
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("The name `{name}` is defined multiple times")]
    NameDefinedMultipleTimes { name: String, span: SourceSpan },

    #[error("Duplicate `[{name}]` attribute")]
    #[diagnostic(help(
        "the `[{name}]` attribute cannot be applied multiple times to the same item"
    ))]
    DuplicateAttributeError {
        name: &'static str,
        #[label("Here")]
        span: SourceSpan,
    },
    #[error("conflicting attributes `[{a}]` and `[{b}]`")]
    #[diagnostic(help(
        "`[{a}]` and `[{b}]` are mutually exclusive. Remove one of the conflicting attributes"
    ))]
    ConflictingAttributes { a: &'static str, b: &'static str },

    #[error("unknown attribute `{name}`")]
    UnknownAttribute {
        name: String,
        #[label("Here")]
        span: SourceSpan,
    },
}
