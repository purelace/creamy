use binrw::binrw;
use compiler_utils::{List, strpool::StringId};

use crate::model::types::{Message, Type};

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            "ExclusiveWrite" => Access::ExclusiveWrite,
            "MultipleWrite" => Access::MultipleWrite,
            _ => Access::Error,
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

    pub const fn name(&self) -> StringId {
        self.name
    }

    pub const fn version(&self) -> u8 {
        self.version
    }

    pub const fn access(&self) -> Access {
        self.access
    }

    pub fn types(&self) -> &[Type] {
        self.types.as_slice()
    }

    pub fn messages(&self) -> &[Message] {
        self.messages.as_slice()
    }
}
