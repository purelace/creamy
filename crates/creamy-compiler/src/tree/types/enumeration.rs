use std::collections::HashSet;

use compiler_utils::{List, strpool::StringPool};
use roxmltree::{Node, NodeType};

use crate::{
    model::types::{CustomType, Enumeration, Type},
    table::TypeTable,
};

#[derive(Debug)]
pub struct EnumToken {
    name: String,
    variants: Vec<String>,
}

impl EnumToken {
    pub fn new(node: Node) -> Self {
        assert_eq!(node.tag_name().name(), "enum");

        let name = node
            .attribute("name")
            .expect("<enum>: missing 'name' attribute")
            .to_string();

        let variants = node
            .children()
            .filter(|node| node.node_type() == NodeType::Element)
            .map(|node| {
                assert_eq!(node.tag_name().name(), "variant");

                node.attribute("name")
                    .expect("<variant>: missing 'name' attribute")
                    .to_string()
            })
            .collect::<Vec<_>>();

        Self { name, variants }
    }

    pub fn resolve(self, _: &TypeTable, pool: &mut StringPool) -> Type {
        let mut names = HashSet::new();

        let name = pool.get_id(&self.name);

        let mut result = List::with_capacity(self.variants.len());
        for variant in &self.variants {
            if !names.insert(variant) {
                panic!("Cannot resolve enum type. Duplicate variant: {variant}");
            }

            result.push(pool.get_id(variant));
        }

        Type::Custom(CustomType::Enum(Enumeration::new(name, result)))
    }
}
