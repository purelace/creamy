mod enumeration;
mod message;
mod numeric;
mod protocol;
mod structure;

use binrw::binrw;
use compiler_utils::strpool::StringId;
use enum_dispatch::enum_dispatch;

pub use enumeration::Enumeration;
pub use message::Message;
pub use numeric::NumericType;
pub use protocol::{Access, Protocol};
pub use structure::Structure;

use crate::{
    model::{Layout, ResolvedType},
    table::{TypeId, TypeTable},
};

#[binrw]
#[enum_dispatch(ResolvedType, Layout)]
#[derive(Debug, Clone)]
pub enum Type {
    Builtin(BuiltinType),
    Custom(CustomType),
}

#[binrw]
#[derive(Debug, Clone, Copy)]
pub enum BuiltinType {
    Numeric(NumericType),
    Array(NumericType, u32),
}

impl Layout for BuiltinType {
    fn size_of(&self, tt: &TypeTable) -> usize {
        match self {
            BuiltinType::Numeric(ty) => ty.size_of(tt),
            BuiltinType::Array(ty, count) => ty.size_of(tt) * *count as usize,
        }
    }

    fn align_of(&self, tt: &TypeTable) -> usize {
        match self {
            BuiltinType::Numeric(ty) => ty.align_of(tt),
            BuiltinType::Array(ty, _) => ty.align_of(tt),
        }
    }
}

impl ResolvedType for BuiltinType {
    fn name(&self) -> StringId {
        match self {
            BuiltinType::Numeric(ty) => ty.name(),
            BuiltinType::Array(_, _) => unreachable!(),
        }
    }
}

#[binrw]
#[enum_dispatch(ResolvedType, Layout)]
#[derive(Debug, Clone)]
pub enum CustomType {
    Struct(Structure),
    Enum(Enumeration),
}

#[binrw]
#[derive(Debug, Clone, Copy)]
pub enum FieldType {
    Type(TypeId),
    Array(TypeId, u32),
}

impl Layout for FieldType {
    fn size_of(&self, tt: &TypeTable) -> usize {
        match self {
            FieldType::Type(ty) => tt.size_of_type(*ty),
            FieldType::Array(ty, size) => tt.size_of_type(*ty) * *size as usize,
        }
    }

    fn align_of(&self, tt: &TypeTable) -> usize {
        match self {
            FieldType::Type(ty) => tt.align_of_type(*ty),
            FieldType::Array(ty, _) => tt.align_of_type(*ty),
        }
    }
}

#[binrw]
#[derive(Debug, Clone, Copy)]
pub struct Field {
    name: StringId,
    kind: FieldType,
}

impl Field {
    pub const fn new(name: StringId, kind: FieldType) -> Self {
        Self { name, kind }
    }
}

impl Layout for Field {
    fn size_of(&self, tt: &TypeTable) -> usize {
        self.kind.size_of(tt)
    }

    fn align_of(&self, tt: &TypeTable) -> usize {
        self.kind.align_of(tt)
    }
}
