use core::{fmt::Debug, mem::MaybeUninit};

use creamy_utils::strpool::StringId;
use hashbrown::HashMap;
use strum::EnumCount;

use crate::{
    error::{Fallback, SemanticError},
    symbols::{NumericSymbol, T_U8_ID, Type, U8_META, U16_META, U32_META, U64_META, U128_META},
    utils::{Align, Array, Size},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TypeMeta {
    align: Align,
    size: Size,
}

impl Fallback for TypeMeta {
    fn fallback() -> Self {
        Self::new(Size::B1, Align::B1)
    }
}

impl TypeMeta {
    /// * Size:  1 byte
    /// * Align: 1 byte
    pub const S1B_A1B: Self = TypeMeta::new_const(1, 1);

    /// * Size:  2 byte
    /// * Align: 2 byte
    pub const S2B_A2B: Self = TypeMeta::new_const(2, 2);

    /// * Size:  4 byte
    /// * Align: 4 byte
    pub const S4B_A4B: Self = TypeMeta::new_const(4, 4);

    /// * Size:  8 byte
    /// * Align: 8 byte
    pub const S8B_A8B: Self = TypeMeta::new_const(8, 8);

    /// * Size:  16 byte
    /// * Align: 16 byte
    pub const S16B_A16B: Self = TypeMeta::new_const(16, 16);

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
    /// * align != 2^k
    /// * align > 16
    pub fn try_new(size: u64, align: u64) -> Result<Self, SemanticError> {
        if !align.is_power_of_two() {
            return Err(SemanticError::AlignIsNotPowerOfTwo { actual: align });
        }

        let Some(align) = Align::new(align) else {
            return Err(SemanticError::ForbiddenAlign { actual: align });
        };

        let size = Size::try_new(size)?;

        Ok(Self { align, size })
    }

    const fn new_const(size: u64, align: u64) -> Self {
        Self {
            align: Align::new(align).unwrap(),
            size: unsafe { Size::new_unchecked(size) },
        }
    }

    pub(crate) const fn new(size: Size, align: Align) -> Self {
        Self { align, size }
    }

    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    #[must_use]
    pub const fn align(self) -> Align {
        self.align
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    #[must_use]
    pub fn size(self) -> Size {
        self.meta.size()
    }

    #[must_use]
    pub const fn align(self) -> Align {
        self.meta.align()
    }
}

impl Fallback for TypeId {
    fn fallback() -> Self {
        T_U8_ID
    }
}

pub struct SymbolTable {
    index: u16,
    types: Array<MaybeUninit<Type>>,
    //Fix 'same Id's but different types'
    inner: HashMap<StringId, TypeId>,
}

impl SymbolTable {
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
        let ident = symbol.ident();
        self.types[index as usize] = MaybeUninit::new(symbol);
        self.inner.insert(ident, TypeId { index, meta });

        self.index += 1;
    }

    #[must_use]
    pub fn contains_name(&self, name: StringId) -> bool {
        self.inner.contains_key(&name)
    }

    #[must_use]
    pub fn finish(self) -> FinishedSymbolTable {
        debug_assert_eq!(self.index as usize, self.types.len());
        FinishedSymbolTable {
            types: unsafe { self.types.assume_init() },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FinishedSymbolTable {
    types: Array<Type>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl FinishedSymbolTable {
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
}
