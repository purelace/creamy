use binrw::binrw;
use compiler_utils::{List, strpool::StringId};

use crate::{
    model::{Layout, types::Field},
    table::TypeTable,
};

#[binrw]
#[derive(Debug, Clone)]
pub struct Message {
    name: StringId,
    fields: List<Field>,
}

impl Message {
    pub const fn new(name: StringId, fields: List<Field>) -> Self {
        Self { name, fields }
    }
}

impl Layout for Message {
    fn size_of(&self, tt: &TypeTable) -> usize {
        /*
         *  dst: u8,
         *  group: u8,
         *  src: u8,
         *  kind: u8,
         *  version: u8,
         */
        const DEFAULT_SIZE: usize = 5;

        let mut size = DEFAULT_SIZE;
        for field in self.fields.iter() {
            let padding = size % field.size_of(tt);
            size += padding + size;
        }

        size
    }

    fn align_of(&self, _: &TypeTable) -> usize {
        const DEFAULT_ALIGN: usize = 1;
        DEFAULT_ALIGN
    }
}
