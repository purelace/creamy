use crate::protocol::{
    Result, definitions::field::FieldDefinition, table::TypeTable, types::ResolvedType,
};

#[derive(Debug, Clone)]
pub struct StructDefinition {
    name: String,
    fields: Vec<FieldDefinition>,
}

impl StructDefinition {
    pub fn can_resolve(&self, _table: &TypeTable) -> bool {
        false
    }

    pub fn resolve(self, _table: &TypeTable) -> ResolvedType {
        todo!()
    }

    pub fn get_reason(&self, _table: &TypeTable) -> Result<()> {
        unreachable!()
    }
}
