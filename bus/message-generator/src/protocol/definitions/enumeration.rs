use quote::format_ident;
use roxmltree::Node;

use crate::protocol::{
    Error, Result,
    definitions::{has_duplicates, read_attribute},
    resolved::Enum,
    table::TypeTable,
    types::ResolvedType,
};

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    name: String,
    variants: Vec<String>,
}

impl EnumDefinition {
    pub fn parse(node: Node) -> Result<EnumDefinition> {
        let name = read_attribute(&node, "name")?;
        let mut variants = Vec::new();

        for node in node.children().filter(|n| !n.is_text()) {
            match node.tag_name().name() {
                "Variant" => {
                    variants.push(read_attribute(&node, "name")?);
                }
                other => return Err(Error::UnknownTag(other.to_string())),
            }
        }

        Ok(EnumDefinition { name, variants })
    }

    pub fn can_resolve(&self, table: &TypeTable) -> bool {
        !table.contains(&self.name) && has_duplicates(&self.variants).is_none()
    }

    fn get_size(&self) -> usize {
        let variants = self.variants.len();
        if variants > u8::MAX as usize {
            2
        } else if variants > u16::MAX as usize {
            4
        } else if variants > u32::MAX as usize {
            8
        } else {
            1
        }
    }

    pub fn resolve(self, _table: &TypeTable) -> ResolvedType {
        let size = self.get_size();
        ResolvedType::Enum(Enum {
            name: format_ident!("{}", self.name),
            variants: self
                .variants
                .into_iter()
                .map(|v| format_ident!("{v}"))
                .collect(),
            size,
        })
    }

    pub fn get_reason(&self, table: &TypeTable) -> Result<()> {
        if table.contains(&self.name) {
            return Err(Error::AlreadyDefined(self.name.clone()));
        }

        if let Some(dup) = has_duplicates(&self.variants) {
            return Err(Error::DuplicateVariant {
                duplicate: dup.to_string(),
                src: self.name.clone(),
            });
        }

        Ok(())
    }
}
