use std::collections::HashSet;

use compiler_utils::{
    List,
    strpool::{StringId, StringPool},
};
use roxmltree::{Node, NodeType};

use crate::{
    StringPoolIntern, model::types::Message, table::TypeTable, tree::types::field::FieldToken,
};

#[derive(Debug)]
enum NestedMessageToken {
    Field(FieldToken),
    Remainder,
}

#[derive(Debug)]
pub struct MessageToken {
    name: StringId,
    fields: Vec<FieldToken>,
}

impl MessageToken {
    pub fn new(node: Node, pool: &mut StringPool) -> Self {
        assert_eq!(node.tag_name().name(), "message");

        let name = node
            .attribute("name")
            .expect("<message>: missing 'name' attribute")
            .intern(pool);

        let fields = node
            .children()
            .filter(|node| node.node_type() == NodeType::Element)
            .map(|n| FieldToken::new(n, pool))
            .collect::<Vec<_>>();

        Self { name, fields }
    }

    pub fn resolve(mut self, tt: &TypeTable, pool: &mut StringPool) -> Message {
        let mut names = HashSet::new();
        let mut fields = List::with_capacity(self.fields.len());
        for field in self.fields.drain(..) {
            let field_name = pool.get_string(field.name());
            if !names.insert(field_name) {
                panic!(
                    "Cannot resolve message type. Duplicate field: {}",
                    field_name
                );
            }

            fields.push(field.resolve(tt));
        }

        Message::new(self.name, fields)
    }
}
