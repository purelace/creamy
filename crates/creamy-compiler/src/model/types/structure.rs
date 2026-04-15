use binrw::binrw;
use compiler_utils::{List, strpool::StringId};

use crate::{
    model::{Layout, ResolvedType, types::Field},
    table::TypeTable,
};

#[binrw]
#[derive(Debug, Clone)]
pub struct Structure {
    name: StringId,
    fields: List<Field>,
}

impl Structure {
    pub const fn new(name: StringId, fields: List<Field>) -> Self {
        Self { name, fields }
    }
}

impl Layout for Structure {
    fn size_of(&self, tt: &TypeTable) -> usize {
        let mut size = 0;
        for field in self.fields.iter() {
            let padding = size % field.size_of(tt);
            size += padding + field.size_of(tt);
        }
        size
    }

    fn align_of(&self, tt: &TypeTable) -> usize {
        self.fields.iter().map(|f| f.align_of(tt)).max().unwrap()
    }
}

impl ResolvedType for Structure {
    fn name(&self) -> StringId {
        self.name
    }
}
