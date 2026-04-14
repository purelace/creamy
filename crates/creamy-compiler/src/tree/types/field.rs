use compiler_utils::strpool::StringPool;
use roxmltree::Node;

use crate::{model::types::Field, table::TypeTable};

#[derive(Debug)]
pub struct FieldToken {
    name: String,
    kind: String,
}

impl FieldToken {
    pub fn new(node: Node) -> Self {
        assert_eq!(node.tag_name().name(), "field");

        let name = node
            .attribute("name")
            .expect("<field>: missing 'name' attribute")
            .to_string();

        let kind = node
            .attribute("type")
            .expect("<field>: missing 'type' attribute")
            .to_string();

        Self { name, kind }
    }

    pub fn resolve(self, tt: &TypeTable, pool: &mut StringPool) -> Field {
        let name = pool.get_id(&self.name);
        let ty_name = pool.get_id(&self.kind);
        let ty = tt.get_type_by_name(ty_name);
        Field::new(name, ty)
    }

    pub const fn name(&self) -> &str {
        self.name.as_str()
    }
}
