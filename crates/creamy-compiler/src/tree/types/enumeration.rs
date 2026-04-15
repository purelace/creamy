use std::collections::HashSet;

use compiler_utils::{
    List,
    strpool::{StringId, StringPool},
};
use roxmltree::{Node, NodeType};

use crate::{
    StringPoolIntern,
    model::types::{CustomType, Enumeration, Type},
    table::TypeTable,
};

#[derive(Debug)]
pub struct EnumToken {
    name: StringId,
    variants: Vec<StringId>,
}

impl EnumToken {
    pub fn new(node: Node, pool: &mut StringPool) -> Self {
        assert_eq!(node.tag_name().name(), "enum");

        let name = node
            .attribute("name")
            .expect("<enum>: missing 'name' attribute")
            .intern(pool);

        let variants = node
            .children()
            .filter(|node| node.node_type() == NodeType::Element)
            .map(|node| {
                assert_eq!(node.tag_name().name(), "variant");

                node.attribute("name")
                    .expect("<variant>: missing 'name' attribute")
                    .intern(pool)
            })
            .collect::<Vec<_>>();

        Self { name, variants }
    }

    pub fn resolve(self, _: &TypeTable, pool: &StringPool) -> Type {
        let mut names = HashSet::new();

        let mut result = List::with_capacity(self.variants.len());
        for variant in &self.variants {
            let variant_name = pool.get_string(*variant);
            if !names.insert(variant_name) {
                panic!("Cannot resolve enum type. Duplicate variant: {variant_name}");
            }

            result.push(*variant);
        }

        Type::Custom(CustomType::Enum(Enumeration::new(self.name, result)))
    }
}
