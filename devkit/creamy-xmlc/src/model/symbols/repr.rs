#![cfg_attr(coverage_nightly, coverage(off))]
use std::fmt::Display;

use binrw::{BinRead, BinWrite};

use crate::{VariantValue, error::SemanticError, model::symbols::NumericSymbol};

#[derive(BinRead, BinWrite, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveRepr {
    #[brw(magic = 0u8)]
    #[default]
    U8,
    #[brw(magic = 1u8)]
    U16,
    #[brw(magic = 2u8)]
    U32,
    #[brw(magic = 3u8)]
    U64,
    #[brw(magic = 4u8)]
    I8,
    #[brw(magic = 5u8)]
    I16,
    #[brw(magic = 6u8)]
    I32,
    #[brw(magic = 7u8)]
    I64,
}

impl PrimitiveRepr {
    #[must_use]
    pub const fn as_numberic_symbol(self) -> NumericSymbol {
        match self {
            PrimitiveRepr::U8 => NumericSymbol::U8,
            PrimitiveRepr::U16 => NumericSymbol::U16,
            PrimitiveRepr::U32 => NumericSymbol::U32,
            PrimitiveRepr::U64 => NumericSymbol::U64,
            PrimitiveRepr::I8 => NumericSymbol::I8,
            PrimitiveRepr::I16 => NumericSymbol::I16,
            PrimitiveRepr::I32 => NumericSymbol::I32,
            PrimitiveRepr::I64 => NumericSymbol::I64,
        }
    }

    #[must_use]
    pub const fn get_min(self) -> i64 {
        match self {
            PrimitiveRepr::U8 | PrimitiveRepr::U16 | PrimitiveRepr::U32 | PrimitiveRepr::U64 => 0,
            PrimitiveRepr::I8 => i8::MIN as i64,
            PrimitiveRepr::I16 => i16::MIN as i64,
            PrimitiveRepr::I32 => i32::MIN as i64,
            PrimitiveRepr::I64 => i64::MIN,
        }
    }

    #[must_use]
    pub const fn get_max(self) -> u64 {
        match self {
            PrimitiveRepr::I8 => i8::MAX as u64,
            PrimitiveRepr::I16 => i16::MAX as u64,
            PrimitiveRepr::I32 => i32::MAX as u64,
            PrimitiveRepr::I64 => i64::MAX as u64,
            PrimitiveRepr::U8 => u8::MAX as u64,
            PrimitiveRepr::U16 => u16::MAX as u64,
            PrimitiveRepr::U32 => u32::MAX as u64,
            PrimitiveRepr::U64 => u64::MAX,
        }
    }

    #[must_use]
    pub fn is_valid_value(self, value: VariantValue) -> bool {
        match value {
            VariantValue::Singed(s) => match self {
                PrimitiveRepr::U8
                | PrimitiveRepr::U16
                | PrimitiveRepr::U32
                | PrimitiveRepr::U64 => false,
                PrimitiveRepr::I8 => (i64::from(i8::MIN)..=i64::from(i8::MAX)).contains(&s),
                PrimitiveRepr::I16 => (i64::from(i16::MIN)..=i64::from(i16::MAX)).contains(&s),
                PrimitiveRepr::I32 => (i64::from(i32::MIN)..=i64::from(i32::MAX)).contains(&s),
                PrimitiveRepr::I64 => (i64::MIN..=i64::MAX).contains(&s),
            },
            VariantValue::Unsigned(u) => match self {
                PrimitiveRepr::U8 => (0..=u64::from(u8::MAX)).contains(&u),
                PrimitiveRepr::U16 => (0..=u64::from(u16::MAX)).contains(&u),
                PrimitiveRepr::U32 => (0..=u64::from(u32::MAX)).contains(&u),
                PrimitiveRepr::U64 => (0..=u64::MAX).contains(&u),

                PrimitiveRepr::I8 => i8::try_from(u).is_ok(),
                PrimitiveRepr::I16 => i16::try_from(u).is_ok(),
                PrimitiveRepr::I32 => i32::try_from(u).is_ok(),
                PrimitiveRepr::I64 => i64::try_from(u).is_ok(),
            },
        }
    }

    #[must_use]
    pub const fn is_signed(self) -> bool {
        match self {
            PrimitiveRepr::U8 | PrimitiveRepr::U16 | PrimitiveRepr::U32 | PrimitiveRepr::U64 => {
                false
            }
            PrimitiveRepr::I8 | PrimitiveRepr::I16 | PrimitiveRepr::I32 | PrimitiveRepr::I64 => {
                true
            }
        }
    }
}

impl TryFrom<&str> for PrimitiveRepr {
    type Error = SemanticError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            _ => Err(SemanticError::InvalidEnumUnderlyingType),
        }
    }
}

impl Display for PrimitiveRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveRepr::U8 => write!(f, "u8"),
            PrimitiveRepr::U16 => write!(f, "u16"),
            PrimitiveRepr::U32 => write!(f, "u32"),
            PrimitiveRepr::U64 => write!(f, "u64"),
            PrimitiveRepr::I8 => write!(f, "i8"),
            PrimitiveRepr::I16 => write!(f, "i16"),
            PrimitiveRepr::I32 => write!(f, "i32"),
            PrimitiveRepr::I64 => write!(f, "i64"),
        }
    }
}
