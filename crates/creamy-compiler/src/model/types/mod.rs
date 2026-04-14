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
    Field(StringId),
}

impl Layout for BuiltinType {
    fn size_of(&self, tt: &TypeTable) -> usize {
        match self {
            BuiltinType::Numeric(ty) => ty.size_of(tt),
            BuiltinType::Array(ty, count) => ty.size_of(tt) * *count as usize,
            BuiltinType::Field(_) => 0,
        }
    }

    fn align_of(&self, tt: &TypeTable) -> usize {
        match self {
            BuiltinType::Numeric(ty) => ty.align_of(tt),
            BuiltinType::Array(ty, _) => ty.align_of(tt),
            BuiltinType::Field(_) => 0,
        }
    }
}

impl ResolvedType for BuiltinType {
    fn name(&self) -> StringId {
        unreachable!()
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
pub struct Field {
    name: StringId,
    kind: TypeId,
}

impl Field {
    pub const fn new(name: StringId, ty: TypeId) -> Self {
        Self { name, kind: ty }
    }
}

impl Layout for Field {
    fn size_of(&self, tt: &TypeTable) -> usize {
        tt.size_of_type(self.kind)
    }

    fn align_of(&self, tt: &TypeTable) -> usize {
        tt.align_of_type(self.kind)
    }
}
