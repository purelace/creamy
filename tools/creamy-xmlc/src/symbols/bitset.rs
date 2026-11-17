use creamy_utils::strpool::StringId;

use crate::table::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitsetValueSymbol {
    pub(crate) ident: StringId,
    pub(crate) repr: TypeId,
    pub(crate) width: u8,
}

impl BitsetValueSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }

    #[must_use]
    pub const fn repr(&self) -> TypeId {
        self.repr
    }

    #[must_use]
    pub const fn width(&self) -> u8 {
        self.width
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitsetSymbol {
    pub(crate) ident: StringId,
    pub(crate) values: Box<[BitsetValueSymbol]>,
}

impl BitsetSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }

    #[must_use]
    pub const fn values(&self) -> &[BitsetValueSymbol] {
        &self.values
    }
}
