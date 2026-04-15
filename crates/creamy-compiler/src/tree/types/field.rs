use compiler_utils::strpool::{StringId, StringPool};
use roxmltree::Node;

use crate::{
    model::types::{Field, FieldType},
    table::TypeTable,
    utils::StringPoolIntern,
};

fn parse_array(input: &str) -> Option<(&str, u32)> {
    let s = input.trim();

    if !s.starts_with('[') || !s.ends_with(']') {
        return None;
    }

    let content = &s[1..s.len() - 1];
    let parts: Vec<&str> = content.split(';').collect();

    if parts.len() != 2 {
        return None;
    }

    let type_ident = parts[0].trim();
    let count = parts[1].trim().parse::<u32>().ok()?;

    Some((type_ident, count))
}

fn is_remainder_type(input: &str) -> bool {
    let s = input.trim();

    if s.starts_with('{') && s.ends_with('}') {
        return s[1..s.len() - 1].trim() == "remainder";
    }

    false
}

#[derive(Debug, Clone, Copy)]
enum FieldTypeToken {
    Type(StringId),
    Array(StringId, u32),
}

impl FieldTypeToken {
    fn new(string: &str, pool: &mut StringPool) -> Self {
        if let Some((name, size)) = parse_array(string) {
            let name = pool.get_id(name);
            FieldTypeToken::Array(name, size)
        } else {
            FieldTypeToken::Type(pool.get_id(string))
        }
    }
}

#[derive(Debug)]
pub struct FieldToken {
    name: StringId,
    kind: FieldTypeToken,
}

impl FieldToken {
    pub fn new(node: Node, pool: &mut StringPool) -> Self {
        assert_eq!(node.tag_name().name(), "field");

        let name = node
            .attribute("name")
            .expect("<field>: missing 'name' attribute")
            .intern(pool);

        let kind = node
            .attribute("type")
            .expect("<field>: missing 'type' attribute");
        let kind = FieldTypeToken::new(kind, pool);

        Self { name, kind }
    }

    pub fn can_resolve(&self, tt: &TypeTable) -> bool {
        match &self.kind {
            FieldTypeToken::Type(id) => tt.get_type_by_name(*id).is_some(),
            FieldTypeToken::Array(id, _) => tt.get_type_by_name(*id).is_some(),
        }
    }

    pub fn resolve(self, tt: &TypeTable) -> Field {
        let kind = match self.kind {
            FieldTypeToken::Type(id) => FieldType::Type(tt.get_type_by_name(id).unwrap()),
            FieldTypeToken::Array(id, size) => {
                FieldType::Array(tt.get_type_by_name(id).unwrap(), size)
            }
        };

        Field::new(self.name, kind)
    }

    pub const fn name(&self) -> StringId {
        self.name
    }
}
