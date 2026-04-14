use std::collections::HashSet;

use compiler_utils::{List, strpool::StringPool};
use roxmltree::{Node, NodeType};

use crate::{
    model::types::{CustomType, Structure, Type},
    table::TypeTable,
    tree::types::field::FieldToken,
};

#[derive(Debug)]
pub struct StructToken {
    name: String,
    fields: Vec<FieldToken>,
}

impl StructToken {
    pub fn new(node: Node) -> Self {
        assert_eq!(node.tag_name().name(), "struct");

        let name = node
            .attribute("name")
            .expect("<struct>: missing 'name' attribute")
            .to_string();

        let fields = node
            .children()
            .filter(|node| node.node_type() == NodeType::Element)
            .map(FieldToken::new)
            .collect::<Vec<_>>();

        Self { name, fields }
    }

    pub fn resolve(mut self, tt: &TypeTable, pool: &mut StringPool) -> Type {
        let name = pool.get_id(&self.name);

        let mut names = HashSet::new();
        let mut fields = List::with_capacity(self.fields.len());
        for field in self.fields.drain(..) {
            if !names.insert(field.name().to_string()) {
                panic!(
                    "Cannot resolve struct type. Duplicate field: {}",
                    field.name()
                );
            }

            fields.push(field.resolve(tt, pool));
        }

        Type::Custom(CustomType::Struct(Structure::new(name, fields)))
    }
}
