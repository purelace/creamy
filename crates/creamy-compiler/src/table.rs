use binrw::binrw;
use compiler_utils::strpool::StringId;
use std::collections::HashMap;

use crate::model::{Layout, types::Type};

#[binrw]
#[derive(Debug, Clone, Copy)]
pub struct TypeId(u32);

#[derive(Default)]
pub struct TypeTable {
    types: Vec<Type>,
    inner: HashMap<StringId, TypeId>,
}

impl TypeTable {
    pub fn get_type_by_name(&self, name: StringId) -> Option<TypeId> {
        self.inner.get(&name).copied()
    }

    pub fn register_type(&mut self, name: StringId, ty: Type) {
        let id = TypeId(self.types.len() as u32);
        self.types.push(ty);
        assert!(self.inner.insert(name, id).is_none());
    }

    pub fn size_of_type(&self, ty: TypeId) -> usize {
        self.types[ty.0 as usize].size_of(self)
    }

    pub fn align_of_type(&self, ty: TypeId) -> usize {
        self.types[ty.0 as usize].align_of(self)
    }

    pub fn reset(&mut self) -> Vec<Type> {
        let result = std::mem::take(&mut self.types);
        self.inner.clear();
        result
    }
}
