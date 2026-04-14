use binrw::binrw;
use compiler_utils::{List, strpool::StringId};

use crate::{
    model::{Layout, ResolvedType},
    table::TypeTable,
};

#[binrw]
#[derive(Debug, Clone)]
pub struct Enumeration {
    name: StringId,
    variants: List<StringId>,
}

impl Enumeration {
    pub const fn new(name: StringId, variants: List<StringId>) -> Self {
        Self { name, variants }
    }
}

impl Layout for Enumeration {
    fn size_of(&self, _: &TypeTable) -> usize {
        let count = self.variants.len();
        if count <= 256 {
            1
        } else if count <= 65536 {
            2
        } else if count <= 4294967296 {
            4
        } else {
            8
        }
    }

    fn align_of(&self, tt: &TypeTable) -> usize {
        self.size_of(tt)
    }
}

impl ResolvedType for Enumeration {
    fn name(&self) -> StringId {
        self.name
    }
}
