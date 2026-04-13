use binrw::binrw;

use crate::intern::StringId;

#[derive(Debug, Clone)]
pub struct Enumeration {
    name: StringId,
    variants: Vec<StringId>,
}
