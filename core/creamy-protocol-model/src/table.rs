use core::{fmt::Debug, mem::MaybeUninit, ops::Index};

use creamy_binary::{BinRead, BinWrite};
use creamy_utils::strpool::{StringId, StringPool};
use hashbrown::HashMap;
use strum::EnumCount;

use crate::{
    error::{Fallback, SemanticError},
    symbols::{NumericSymbol, Type, U8_META, U16_META, U32_META, U64_META, U128_META},
    utils::{Align, Array, Range, Size, TypesRange},
};

#[derive(BinRead, BinWrite, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeMeta {
    /// See [Align] for valid values
    /// * 00000 - size (max 28) `[11100_XXX]`
    /// * 000   - align (max 3) `[XXXXX_011]`
    value: u8,
}

impl Debug for TypeMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TypeMeta")
            .field("size", &self.size().value())
            .field("align", &self.align().size())
            .finish()
    }
}

impl Fallback for TypeMeta {
    fn fallback() -> Self {
        Self::new(Size::B1, Align::B1)
    }
}

impl TypeMeta {
    /// * Size:  1 byte
    /// * Align: 1 byte
    pub const S1B_A1B: Self = TypeMeta::new_raw_with_assert(1, 0);

    /// * Size:  2 byte
    /// * Align: 2 byte
    pub const S2B_A2B: Self = TypeMeta::new_raw_with_assert(2, 1);

    /// * Size:  4 byte
    /// * Align: 4 byte
    pub const S4B_A4B: Self = TypeMeta::new_raw_with_assert(4, 2);

    /// * Size:  8 byte
    /// * Align: 8 byte
    pub const S8B_A8B: Self = TypeMeta::new_raw_with_assert(8, 3);

    /// * Size:  16 byte
    /// * Align: 16 byte
    pub const S16B_A16B: Self = TypeMeta::new_raw_with_assert(16, 4);

    /// Tries to wrap raw size and alignment values into a safe object.
    ///
    /// Allowed values:
    /// * Size: 0 < ``size`` <= 28
    /// * Align: ``[1, 2, 4, 8, 16]``
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// * size == 0
    /// * size > [`Size::MAX_VALUE`]
    /// * align != 2^k
    /// * align > 16
    pub const fn try_new(size: u8, align: u8) -> Result<Self, SemanticError> {
        if size == 0 || size > Size::MAX_VALUE {
            return Err(SemanticError::InvalidSize {
                actual: size as usize,
            });
        }

        if align > 16 {
            return Err(SemanticError::ForbiddenAlign { actual: align });
        }

        if !align.is_power_of_two() {
            return Err(SemanticError::AlignIsNotPowerOfTwo { actual: align });
        }

        let size = size << 3;
        let value = size | Align::discriminant_from_size(align);
        Ok(Self { value })
    }

    const fn new_raw_with_assert(size: u8, align: u8) -> Self {
        assert!(size <= Size::MAX_VALUE);
        assert!(align <= 4);
        let size = size << 3;
        let value = size | align;
        Self { value }
    }

    pub(crate) const fn new(size: Size, align: Align) -> Self {
        let size = size.value();
        let align = align.discriminant();
        Self::new_raw_with_assert(size, align)
    }

    #[must_use]
    // TODO: const
    pub fn size(self) -> Size {
        cfg_select! {
            debug_assertions =>
            {
                #[allow(clippy::missing_panics_doc)]
                Size::try_new(self.value >> 3).expect("Unreachable!")
            }
            _ => unsafe { Size::new_unchecked(self.value >> 3) },
        }
    }

    #[must_use]
    pub const fn align(self) -> Align {
        let value = self.value & 0b0000_0111;
        unsafe { core::mem::transmute(value) }
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId {
    index: u16,
    meta: TypeMeta,
}

impl TypeId {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(crate) const fn new(id: StringId, meta: TypeMeta) -> Self {
        TypeId {
            index: id.value(),
            meta,
        }
    }

    pub(crate) const fn index(self) -> u16 {
        self.index
    }

    #[must_use]
    pub fn size(self) -> Size {
        self.meta.size()
    }

    #[must_use]
    pub const fn align(self) -> Align {
        self.meta.align()
    }
}

pub struct TypeTable {
    index: u16,
    types: Array<MaybeUninit<Type>>,
    //Fix 'same Id's but different types'
    inner: HashMap<StringId, TypeId>,
}

impl TypeTable {
    #[must_use]
    pub fn new(types: u16) -> Self {
        let types = types as usize + NumericSymbol::COUNT;

        let mut instance = Self {
            index: 0,
            types: Array::zeroed(types),
            inner: HashMap::new(),
        };

        instance.register_type(U8_META, Type::Numeric(NumericSymbol::U8));
        instance.register_type(U16_META, Type::Numeric(NumericSymbol::U16));
        instance.register_type(U32_META, Type::Numeric(NumericSymbol::U32));
        instance.register_type(U64_META, Type::Numeric(NumericSymbol::U64));
        instance.register_type(U128_META, Type::Numeric(NumericSymbol::U128));

        instance.register_type(U8_META, Type::Numeric(NumericSymbol::I8));
        instance.register_type(U16_META, Type::Numeric(NumericSymbol::I16));
        instance.register_type(U32_META, Type::Numeric(NumericSymbol::I32));
        instance.register_type(U64_META, Type::Numeric(NumericSymbol::I64));
        instance.register_type(U128_META, Type::Numeric(NumericSymbol::I128));

        instance.register_type(U32_META, Type::Numeric(NumericSymbol::F32));
        instance.register_type(U64_META, Type::Numeric(NumericSymbol::F64));

        instance
    }

    #[must_use]
    pub fn get_type_by_name(&self, name: StringId) -> Option<TypeId> {
        self.inner.get(&name).copied()
    }

    pub fn register_type(&mut self, meta: TypeMeta, symbol: Type) {
        let index = self.index;
        self.types[index as usize] = MaybeUninit::new(symbol);
        self.inner.insert(symbol.ident(), TypeId { index, meta });

        self.index += 1;
    }

    #[must_use]
    pub fn contains_name(&self, name: StringId) -> bool {
        self.inner.contains_key(&name)
    }

    #[must_use]
    pub fn finish(self) -> FinishedTypeTable {
        debug_assert_eq!(self.index as usize, self.types.len());
        FinishedTypeTable {
            types: unsafe { self.types.assume_init() },
        }
    }
}

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
pub struct FinishedTypeTable {
    types: Array<Type>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl FinishedTypeTable {
    #[must_use]
    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    #[must_use]
    pub const fn types(&self) -> &[Type] {
        self.types.as_slice()
    }

    #[must_use]
    pub fn get_type(&self, id: TypeId) -> &Type {
        &self.types[id.index as usize]
    }

    #[must_use]
    pub fn name_of_type(&self, id: TypeId) -> StringId {
        self.get_type(id).ident()
    }

    pub fn replace_identifiers(&mut self, from: &StringPool, to: &mut StringPool) {
        macro_rules! replace_symbol {
            ($symbol:expr) => {
                let value = from.get_string($symbol.ident());
                let id = to.get_id_or_add(value);
                *$symbol = $symbol.with_ident(id);
            };
        }

        for ty in self.types.iter_mut() {
            replace_symbol!(ty);
        }
    }
}

impl Index<TypesRange> for FinishedTypeTable {
    type Output = [Type];

    fn index(&self, index: TypesRange) -> &Self::Output {
        &self.types[index.as_range()]
    }
}
