use std::collections::HashSet;

use compiler_utils::{List, strpool::StringPool};
use roxmltree::{Node, NodeType};

use crate::{model::types::Message, table::TypeTable, tree::types::field::FieldToken};

#[derive(Debug)]
pub struct MessageToken {
    name: String,
    fields: Vec<FieldToken>,
}

impl MessageToken {
    pub fn new(node: Node) -> Self {
        assert_eq!(node.tag_name().name(), "message");

        let name = node
            .attribute("name")
            .expect("<message>: missing 'name' attribute")
            .to_string();

        let fields = node
            .children()
            .filter(|node| node.node_type() == NodeType::Element)
            .map(FieldToken::new)
            .collect::<Vec<_>>();

        Self { name, fields }
    }

    pub fn resolve(mut self, tt: &TypeTable, pool: &mut StringPool) -> Message {
        let name = pool.get_id(&self.name);

        let mut names = HashSet::new();
        let mut fields = List::with_capacity(self.fields.len());
        for field in self.fields.drain(..) {
            if !names.insert(field.name().to_string()) {
                panic!(
                    "Cannot resolve message type. Duplicate field: {}",
                    field.name()
                );
            }

            fields.push(field.resolve(tt, pool));
        }

        Message::new(name, fields)
    }
}
