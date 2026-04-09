use quote::format_ident;
use roxmltree::Node;

use crate::protocol::{
    Error, Result,
    definitions::{field::FieldDefinition, read_attribute},
    resolved::{ChunkCoder, Pattern, PatternType},
    table::TypeTable,
    types::TypeHandle,
};

#[derive(Debug, Clone)]
pub struct VariableFieldDefinition {
    pub name: String,
    pub length_src: String,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    General(FieldDefinition),
    Variable(VariableFieldDefinition),
}

impl FieldType {
    fn ty(&self) -> &str {
        match self {
            FieldType::General(def) => &def.ty,
            FieldType::Variable(_) => "String",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkPattern {
    name: String,
    fields: Vec<FieldType>,
}

impl ChunkPattern {
    pub fn parse(src_name: String, node: Node) -> Result<ChunkPattern> {
        let mut fields = Vec::new();
        for node in node.children().filter(|n| !n.is_text()) {
            let tag_name = node.tag_name().name();
            assert_eq!(tag_name, "Pattern", "Unknown '{tag_name}' tag");

            let name = read_attribute(&node, "name")?;
            let name = if name == "type" {
                "ty".to_string()
            } else {
                name
            };
            let ty = read_attribute(&node, "type")?;
            let field = if ty == "String" {
                let length_src = read_attribute(&node, "length_src")?;
                FieldType::Variable(VariableFieldDefinition { name, length_src })
            } else {
                FieldType::General(FieldDefinition { name, ty })
            };
            fields.push(field);
        }

        Ok(ChunkPattern {
            name: src_name,
            fields,
        })
    }

    pub fn can_resolve(&self, table: &TypeTable) -> bool {
        self.fields.iter().all(|f| table.get_type(f.ty()).is_some())
    }

    pub fn resolve(mut self, table: &TypeTable) -> ChunkCoder {
        let mut patterns = Vec::new();

        for pattern in self.fields.drain(..) {
            match pattern {
                FieldType::General(def) => {
                    let ty = table.get_type(&def.ty).unwrap();
                    patterns.push(Pattern {
                        name: format_ident!("{}", &def.name.to_lowercase()),
                        ty: PatternType::CompileTime(ty.get_handle()),
                        size: ty.get_size(),
                    });
                }
                FieldType::Variable(var_def) => {
                    let ty = table.get_type("String").unwrap();
                    patterns.push(Pattern {
                        name: format_ident!("{}", &var_def.name.to_lowercase()),
                        ty: PatternType::RunTime(TypeHandle::String, var_def.length_src),
                        size: ty.get_size(),
                    });
                }
            }
        }

        ChunkCoder {
            name: format_ident!("{}Coder", self.name),
            patterns,
        }
    }

    pub fn get_reason(&self, table: &TypeTable) -> Result<()> {
        for field in &self.fields {
            if table.get_type(field.ty()).is_none() {
                return Err(Error::MissingType(field.ty().to_string()));
            }
        }

        Ok(())
    }
}
