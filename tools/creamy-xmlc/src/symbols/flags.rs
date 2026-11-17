use creamy_utils::strpool::StringId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlagsSymbol {
    pub(crate) ident: StringId,
    pub(crate) options: Box<[OptionSymbol]>,
}

impl FlagsSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }

    #[must_use]
    pub const fn options(&self) -> &[OptionSymbol] {
        &self.options
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptionSymbol {
    pub(crate) ident: StringId,
}

impl OptionSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }
}
