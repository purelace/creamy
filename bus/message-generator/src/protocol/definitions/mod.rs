mod chunk;
mod enumeration;
mod field;
mod message;
mod structure;
mod wrapper;

use std::collections::HashSet;

use roxmltree::{Document, Node};

use crate::protocol::{
    Error, Result,
    definitions::{
        enumeration::EnumDefinition, message::MessageDefinition, structure::StructDefinition,
        wrapper::WrapperDefinition,
    },
    table::TypeTable,
    types::ResolvedType,
};

fn has_duplicates(vec: &[String]) -> Option<&str> {
    let mut set = HashSet::with_capacity(vec.len());
    for item in vec {
        if !set.insert(item) {
            return Some(item.as_str());
        }
    }

    None
}

fn read_attribute(node: &Node, attr_name: &str) -> Result<String> {
    let Some(name_attr) = node.attribute(attr_name) else {
        return Err(Error::MissingAttribute {
            name: attr_name.to_string(),
            src: node.tag_name().name().to_string(),
        });
    };
    Ok(name_attr.to_string())
}

fn read_optional_attribute(node: &Node, attr_name: &str) -> Option<String> {
    node.attribute(attr_name).map(str::to_string)
}

#[derive(Debug, Clone)]
pub enum Definition {
    Enum(EnumDefinition),
    Wrapper(WrapperDefinition),
    Structs(StructDefinition),
    Message(MessageDefinition),
}

impl Definition {
    pub fn can_resolve(&self, table: &TypeTable) -> bool {
        match self {
            Definition::Enum(def) => def.can_resolve(table),
            Definition::Wrapper(def) => def.can_resolve(table),
            Definition::Structs(def) => def.can_resolve(table),
            Definition::Message(def) => def.can_resolve(table),
        }
    }

    pub fn resolve(self, table: &TypeTable) -> ResolvedType {
        match self {
            Definition::Enum(def) => def.resolve(table),
            Definition::Wrapper(def) => def.resolve(table),
            Definition::Structs(def) => def.resolve(table),
            Definition::Message(def) => def.resolve(table),
        }
    }

    pub fn get_reason(&self, table: &TypeTable) -> Result<()> {
        match self {
            Definition::Enum(def) => def.get_reason(table),
            Definition::Wrapper(def) => def.get_reason(table),
            Definition::Structs(def) => def.get_reason(table),
            Definition::Message(def) => def.get_reason(table),
        }
    }
}

#[derive(Debug)]
pub struct Definitions {
    pub protocol: String,
    pub definitions: Vec<Definition>,
}

impl Definitions {
    #[allow(dead_code)]
    pub fn parse_from_xml(content: &str) -> Result<Self> {
        let document = Document::parse(content).unwrap();
        let root = document.root();
        let root = root.first_child().unwrap();
        assert_eq!(root.tag_name().name(), "Protocol");

        let name = read_attribute(&root, "name")?;
        let mut definitions = Vec::new();

        for node in root.children().filter(|n| !n.is_text()) {
            match node.tag_name().name() {
                "Enum" => definitions.push(Definition::Enum(EnumDefinition::parse(node)?)),
                "Structs" => {}
                "Wrapper" => definitions.push(Definition::Wrapper(WrapperDefinition::parse(node)?)),
                "Message" => definitions.push(Definition::Message(MessageDefinition::parse(node)?)),
                "LocalKindTable" => {}
                other => panic!("Unknown '{other}' tag"),
            }
        }

        Ok(Self {
            protocol: name,
            definitions,
        })
    }
}
