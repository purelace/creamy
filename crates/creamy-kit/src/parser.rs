use std::collections::HashMap;

use roxmltree::{Node, NodeType};

use crate::{
    intern::{StringId, StringPool},
    types::{Enumeration, Type},
};

#[derive(Debug)]
pub struct RawProtocol {
    name: String,
    version: String,
    access: String,
}

impl RawProtocol {
    fn new(root: Node) -> Self {
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

        Self {
            name,
            version,
            access,
        }
    }
}

#[derive(Debug)]
pub struct RawField {
    name: String,
    kind: String,
}

impl RawField {
    fn new(node: Node) -> Self {
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
}

#[derive(Debug)]
pub struct RawMessage {
    name: String,
    fields: Vec<RawField>,
}

impl RawMessage {
    fn new(node: Node) -> Self {
        assert_eq!(node.tag_name().name(), "message");

        let name = node
            .attribute("name")
            .expect("<message>: missing 'name' attribute")
            .to_string();

        let fields = node
            .children()
            .filter(|node| node.node_type() == NodeType::Element)
            .map(RawField::new)
            .collect::<Vec<_>>();

        Self { name, fields }
    }
}

#[derive(Debug)]
pub struct RawStruct {
    name: String,
    fields: Vec<RawField>,
}

impl RawStruct {
    fn new(node: Node) -> Self {
        assert_eq!(node.tag_name().name(), "struct");

        let name = node
            .attribute("name")
            .expect("<struct>: missing 'name' attribute")
            .to_string();

        let fields = node
            .children()
            .filter(|node| node.node_type() == NodeType::Element)
            .map(RawField::new)
            .collect::<Vec<_>>();

        Self { name, fields }
    }
}

#[derive(Debug)]
pub struct RawEnum {
    name: String,
    variants: Vec<String>,
}

impl RawEnum {
    fn new(node: Node) -> Self {
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
}

#[derive(Debug)]
pub struct ParsedProtocol {
    protocol: RawProtocol,
    messages: Vec<RawMessage>,
    structs: Vec<RawStruct>,
    enums: Vec<RawEnum>,
}

fn parse(content: &str) -> Result<ParsedProtocol, Box<dyn std::error::Error>> {
    let doc = roxmltree::Document::parse(content)?;
    let root = doc.root().first_child().unwrap();
    let protocol = RawProtocol::new(root);

    let mut messages = vec![];
    let mut structs = vec![];
    let mut enums = vec![];

    for node in root
        .children()
        .filter(|node| node.node_type() == NodeType::Element)
    {
        match node.tag_name().name() {
            "message" => {
                messages.push(RawMessage::new(node));
            }
            "struct" => {
                structs.push(RawStruct::new(node));
            }
            "enum" => {
                enums.push(RawEnum::new(node));
            }
            value => panic!("Unsupported '{value}'"),
        }
    }

    Ok(ParsedProtocol {
        protocol,
        messages,
        structs,
        enums,
    })
}

#[derive(Default)]
pub struct TypeTable {
    inner: HashMap<StringId, Type>,
}

fn resolve_types(mut protocol: ParsedProtocol) {
    let mut pool = StringPool::default();
    let mut table = TypeTable::default();

    for e in protocol.enums.drain(..) {
        let id = pool.get_id(&e.name);
        if table.inner.contains_key(&id) {
            panic!(
                "Cannot resolve type: {}. Type with this name already defined",
                e.name
            );
        }
        //let mut variants = Vec::with_capacity(e.variants.len());
        //for variant in e.variants.drain(..) {
        //    let id = pool.get_id(&variant);
        //    if table.inner.contains_key(&id) {
        //        panic!(
        //            "Cannot resolve type: {}. Type with this name already defined",
        //            e.name
        //        );
        //    }
        //}

        //table.inner.insert(id, Enumeration {
        //    name: e.name,
        //    variants: ,
        //})
    }
}

mod tests {
    use crate::parser::parse;

    #[test]
    fn test() {
        const PATH: &str = "/mnt/ssd/fusionwm/creamy/example/plugin/declare/api.xml";
        let content = std::fs::read_to_string(PATH).unwrap();
        let parsed = parse(&content).unwrap();
        dbg!(parsed);
    }
}
