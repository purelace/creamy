use creamy_xmlc::model::{
    TypeId,
    strpool::{StringPool, StringPoolResolver},
    symbols::{
        BitsetValueSymbol, PrimitiveRepr, T_I8_ID, T_I16_ID, T_I32_ID, T_I64_ID, T_I128_ID,
        T_U8_ID, T_U16_ID, T_U32_ID, T_U64_ID,
    },
};

use crate::SymbolIterator;

pub struct EnrichedBitsetSymbol<'s, I>
where
    I: SymbolIterator<EnrichedBitsetValueSymbol<'s>>,
{
    pub name: &'s str,
    pub values: I,
}

#[derive(Clone)]
pub struct BitsetValueList<'a> {
    pool: &'a StringPool,
    slice: &'a [BitsetValueSymbol],
    current_index: usize,

    bits: u8,
}

impl<'a> BitsetValueList<'a> {
    #[must_use]
    pub const fn new(pool: &'a StringPool, slice: &'a [BitsetValueSymbol]) -> Self {
        Self {
            pool,
            slice,
            current_index: 0,
            bits: 0,
        }
    }
}

const fn is_signed_type(t: TypeId) -> bool {
    matches!(t, T_I8_ID | T_I16_ID | T_I32_ID | T_I64_ID | T_I128_ID)
}

fn get_backing_type(type_id: TypeId, width: u8) -> PrimitiveRepr {
    match (is_signed_type(type_id), width) {
        (true, 1) => PrimitiveRepr::I8,
        (true, 2) => PrimitiveRepr::I16,
        (true, 3..=4) => PrimitiveRepr::I32,
        (true, 5..=8) => PrimitiveRepr::I64,
        //(true, 9..=16) => P,
        (false, 1) => PrimitiveRepr::U8,
        (false, 2) => PrimitiveRepr::U16,
        (false, 3..=4) => PrimitiveRepr::U32,
        (false, 5..=8) => PrimitiveRepr::U64,
        //(false, 9..=16) => "u128",
        (_, width) => unreachable!("Unreachable width: {width}"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnrichedBitsetValueSymbol<'s> {
    /// Width of the value in bytes.
    pub bytes: u8,

    /// Width of the value in bits.
    pub bits: u8,

    // TODO: this field has same value like the `backing_type` field
    /// This value indicates the number of bytes to read before applying the shift and/or cast operations
    pub read_window_bytes: u8,

    /// Shift value used to align value bits in valid position for reading
    pub shift: u8,

    pub start_pos: u8,
    pub end_pos: u8,

    #[doc = include_str!("../../docs/bitset.md")]
    pub backing_type: PrimitiveRepr,

    pub repr: PrimitiveRepr,

    /// Name of the value.
    pub name: &'s str,
}

impl<'s> Iterator for BitsetValueList<'s> {
    type Item = EnrichedBitsetValueSymbol<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.slice.len() {
            return None;
        }

        let symbol = self.slice[self.current_index];

        let prev_bits = self.bits;
        self.bits += symbol.bits();
        let read_window_bytes = self.bits.div_ceil(8) - prev_bits.div_euclid(8);

        let bytes = symbol.bits().div_ceil(8);

        let end_pos = self.bits.div_ceil(8);
        let start_pos = end_pos - read_window_bytes;

        let repr = match symbol.repr() {
            T_I8_ID => PrimitiveRepr::I8,
            T_I16_ID => PrimitiveRepr::I16,
            T_I32_ID => PrimitiveRepr::I32,
            T_I64_ID => PrimitiveRepr::I64,
            T_U8_ID => PrimitiveRepr::U8,
            T_U16_ID => PrimitiveRepr::U16,
            T_U32_ID => PrimitiveRepr::U32,
            T_U64_ID => PrimitiveRepr::U64,
            _ => unreachable!(),
        };

        let backing_type = get_backing_type(symbol.repr(), read_window_bytes);

        let backend_width = read_window_bytes * 8;
        let left_pad = (prev_bits - start_pos * 8) + symbol.bits();
        let shift = backend_width - left_pad;
        self.current_index += 1;

        Some(EnrichedBitsetValueSymbol {
            bytes,
            read_window_bytes,
            bits: symbol.bits(),
            shift,
            start_pos,
            end_pos,
            backing_type,
            repr,
            name: symbol.ident().resolve(self.pool),
        })
    }
}

impl<'a> SymbolIterator<EnrichedBitsetValueSymbol<'a>> for BitsetValueList<'a> {}
