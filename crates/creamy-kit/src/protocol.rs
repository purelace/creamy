use std::collections::HashMap;

use binrw::{BinRead, BinWrite, binrw};

use crate::{
    intern::StringId,
    types::{Message, Type},
};

#[binrw]
#[derive(Debug, Clone, Copy)]
pub enum Access {
    #[brw(magic(0u8))]
    Error,
    #[brw(magic(1u8))]
    ExclusiveWrite,
    #[brw(magic(2u8))]
    MultipleWrite,
}

#[derive(Debug)]
pub struct Protocol {
    name: StringId,
    version: u8,
    access: Access,
    messages: Vec<Message>,
    types: Vec<Type>,
}
