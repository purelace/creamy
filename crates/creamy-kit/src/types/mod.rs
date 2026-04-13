mod enumeration;
mod message;
mod numeric;
mod structure;

pub use enumeration::Enumeration;
pub use message::Message;
pub use numeric::NumericType;
pub use structure::Structure;

use crate::intern::StringId;

#[derive(Debug, Clone)]
pub enum Type {
    Builtin(BuiltinType),
    Custom(CustomType),
}

#[derive(Debug, Clone, Copy)]
pub enum BuiltinType {
    Numeric(NumericType),
    Array(NumericType, u32),
    Field(StringId),
}

#[derive(Debug, Clone)]
pub enum CustomType {
    Struct(Structure),
    Enum(Enumeration),
}

#[derive(Debug, Clone)]
pub struct Field {
    name: StringId,
    kind: Type,
}
