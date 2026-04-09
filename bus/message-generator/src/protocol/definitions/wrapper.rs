use quote::format_ident;
use roxmltree::Node;

use crate::protocol::{
    Error, Result, definitions::read_attribute, resolved::Wrapper, table::TypeTable,
    types::ResolvedType,
};

#[derive(Debug, Clone)]
pub struct WrapperDefinition {
    name: String,
    types: Vec<String>,
}

impl WrapperDefinition {
    pub fn parse(node: Node) -> Result<Self> {
        let name = read_attribute(&node, "name")?;
        let mut types = Vec::new();

        for node in node.children().filter(|n| !n.is_text()) {
            match node.tag_name().name() {
                "Type" => {
                    types.push(read_attribute(&node, "name")?);
                }
                other => return Err(Error::UnknownTag(other.to_string())),
            }
        }

        Ok(Self { name, types })
    }

    pub fn can_resolve(&self, table: &TypeTable) -> bool {
        !table.contains(&self.name)
            && self
                .types
                .iter()
                .all(|ty| table.get_primitive(ty).is_some())
    }

    pub fn resolve(mut self, table: &TypeTable) -> ResolvedType {
        let mut types = Vec::new();
        let mut total_size = 0;
        let mut max_align = 0;
        for ty in self.types.drain(..) {
            let Some(primitive) = table.get_primitive(&ty) else {
                unreachable!();
            };

            let size = primitive.get_size();
            let alignment = primitive.get_align();
            let remainder = total_size % alignment;
            max_align = alignment.max(max_align);
            let padding = if remainder == 0 {
                0
            } else {
                let padding = alignment - remainder;
                types.push(ResolvedType::ByteArray(padding).get_handle());
                padding
            };

            total_size += padding + size;
            types.push(primitive.get_handle());
        }

        ResolvedType::Wrapper(Wrapper {
            name: format_ident!("{}", self.name),
            types,
            size: total_size,
            align: max_align,
        })
    }

    pub fn get_reason(&self, _table: &TypeTable) -> Result<()> {
        unreachable!()
    }
}
