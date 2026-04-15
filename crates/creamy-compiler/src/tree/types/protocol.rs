use compiler_utils::strpool::{StringId, StringPool};
use roxmltree::{Node, NodeType};

use crate::{
    StringPoolIntern,
    tree::types::{EnumToken, MessageToken, StructToken},
};

#[derive(Debug)]
pub struct ProtocolTree {
    pub name: StringId,
    pub version: String,
    pub access: String,
    pub structs: Vec<StructToken>,
    pub enums: Vec<EnumToken>,
    pub messages: Vec<MessageToken>,
}

impl ProtocolTree {
    pub fn from_str(
        content: &str,
        pool: &mut StringPool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let doc = roxmltree::Document::parse(content)?;
        let root = doc.root().first_child().unwrap();
        Ok(ProtocolTree::new(root, pool))
    }

    pub fn new(root: Node, pool: &mut StringPool) -> Self {
        assert_eq!(root.tag_name().name(), "protocol");
        let name = root
            .attribute("name")
            .expect("<protocol>: missing 'name' attribute")
            .intern(pool);

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
                    messages.push(MessageToken::new(node, pool));
                }
                "struct" => {
                    structs.push(StructToken::new(node, pool));
                }
                "enum" => {
                    enums.push(EnumToken::new(node, pool));
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
