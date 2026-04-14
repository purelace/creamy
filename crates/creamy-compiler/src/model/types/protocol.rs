use binrw::binrw;
use compiler_utils::{List, strpool::StringId};

use crate::model::types::{Message, Type};

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy)]
pub enum Access {
    #[brw(magic(0u8))]
    Error,
    #[brw(magic(1u8))]
    ExclusiveWrite,
    #[brw(magic(2u8))]
    MultipleWrite,
}

impl Access {
    pub fn from_str(string: &str) -> Access {
        match string {
            "error" => Access::Error,
            "exclusive_write" => Access::ExclusiveWrite,
            "multuple_write" => Access::MultipleWrite,
            _ => panic!("Unknown value"),
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug)]
pub struct Protocol {
    name: StringId,
    version: u8,
    access: Access,
    types: List<Type>,
    messages: List<Message>,
}

impl Protocol {
    pub const fn new(
        name: StringId,
        version: u8,
        access: Access,
        types: List<Type>,
        messages: List<Message>,
    ) -> Self {
        Self {
            name,
            version,
            access,
            types,
            messages,
        }
    }
}
