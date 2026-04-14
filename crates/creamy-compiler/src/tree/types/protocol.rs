use roxmltree::{Node, NodeType};

use crate::tree::types::{EnumToken, MessageToken, StructToken};

#[derive(Debug)]
pub struct ProtocolTree {
    pub name: String,
    pub version: String,
    pub access: String,
    pub structs: Vec<StructToken>,
    pub enums: Vec<EnumToken>,
    pub messages: Vec<MessageToken>,
}

impl ProtocolTree {
    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let doc = roxmltree::Document::parse(content)?;
        let root = doc.root().first_child().unwrap();
        Ok(ProtocolTree::new(root))
    }

    pub fn new(root: Node) -> Self {
        assert_eq!(root.tag_name().name(), "protocol");
        let name = root
            .attribute("name")
            .expect("<protocol>: missing 'name' attribute")
            .to_string();

        let version = root
            .attribute("version")
            .expect("<protocol>: missing 'version' attribute")
            .to_string();

        let access = root
            .attribute("access")
            .expect("<protocol>: missing 'access' attribute")
            .to_string();

        let mut structs = vec![];
        let mut enums = vec![];
        let mut messages = vec![];

        for node in root
            .children()
            .filter(|node| node.node_type() == NodeType::Element)
        {
            match node.tag_name().name() {
                "message" => {
                    messages.push(MessageToken::new(node));
                }
                "struct" => {
                    structs.push(StructToken::new(node));
                }
                "enum" => {
                    enums.push(EnumToken::new(node));
                }
                value => panic!("Unsupported '{value}'"),
            }
        }

        Self {
            name,
            version,
            access,
            structs,
            enums,
            messages,
        }
    }
}
