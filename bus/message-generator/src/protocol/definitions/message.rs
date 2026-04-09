use quote::format_ident;
use roxmltree::{Node, NodeType};

use crate::protocol::{
    Error, Result,
    definitions::{chunk::ChunkPattern, field::FieldDefinition, has_duplicates, read_attribute},
    resolved::{Field, Message},
    table::TypeTable,
    types::{ResolvedType, TypeHandle},
};

#[derive(Debug, Clone)]
pub struct MessageDefinition {
    name: String,
    ty: String,
    version: String,
    fields: Vec<FieldDefinition>,
    chunk: Option<ChunkPattern>,
}

impl MessageDefinition {
    pub fn parse(node: Node) -> Result<Self> {
        assert_eq!(node.node_type(), NodeType::Element);
        let name = read_attribute(&node, "name")?;
        let ty = read_attribute(&node, "type")?;
        let version = read_attribute(&node, "version")?;

        let mut fields = Vec::new();
        let mut chunk = None;

        for node in node.children().filter(|n| !n.is_text()) {
            match node.tag_name().name() {
                "Field" => {
                    let name = read_attribute(&node, "name")?;
                    let ty = read_attribute(&node, "type")?;
                    fields.push(FieldDefinition { name, ty });
                }
                "Chunk" => {
                    assert!(chunk.is_none());
                    chunk = Some(ChunkPattern::parse(name.clone(), node)?);
                }
                other => return Err(Error::UnknownTag(other.to_string())),
            }
        }

        Ok(Self {
            name,
            ty,
            version,
            fields,
            chunk,
        })
    }

    pub fn can_resolve(&self, table: &TypeTable) -> bool {
        !table.contains(&self.name)
            && has_duplicates(
                &self
                    .fields
                    .iter()
                    .map(|m| m.name.clone())
                    .collect::<Vec<_>>(),
            )
            .is_none()
            && self.fields.iter().all(|f| table.get_type(&f.ty).is_some())
            && self.chunk.as_ref().is_none_or(|c| c.can_resolve(table))
    }

    pub fn resolve(mut self, table: &TypeTable) -> ResolvedType {
        const MESSAGE_SIZE: usize = 32;

        //DST (u8) + SRC(u8) + VERSION (u8) + TYPE (u16)
        let mut available_size = MESSAGE_SIZE - (1 + 1 + 1 + 2);
        let mut max_align = 2;
        let mut total_size = 0;
        let mut fields = Vec::new();

        let mut padding_index = 0;
        for field in self.fields.drain(..) {
            let Some(ty) = table.get_type(&field.ty) else {
                unreachable!()
            };

            let ty = match ty {
                ResolvedType::Remainder => ResolvedType::ByteArray(available_size),
                ResolvedType::Variable(_) => todo!(),
                other => other.clone(),
            };

            let size = ty.get_size();
            let field_alignment = ty.get_align();
            max_align = field_alignment.max(max_align);

            let remainder = total_size % field_alignment;
            if remainder != 0 {
                let padding = field_alignment - remainder;
                total_size += padding;
                available_size -= padding;

                fields.push(Field {
                    name: format_ident!("padding{padding_index}"),
                    ty: TypeHandle::ByteArray(padding),
                });

                padding_index += 1;
            }

            total_size += size;
            available_size -= size;

            //TODO overflow

            fields.push(Field {
                name: format_ident!("{}", field.name),
                ty: ty.get_handle(),
            });
        }

        let chunk = if let Some(chunk) = self.chunk {
            fields.push(Field {
                name: format_ident!("chunk"),
                ty: TypeHandle::ByteArray(available_size),
            });
            total_size += available_size;
            available_size = 0;
            Some(chunk.resolve(table))
        } else {
            None
        };

        if available_size != 0 {
            fields.push(Field {
                name: format_ident!("padding{padding_index}"),
                ty: TypeHandle::ByteArray(available_size),
            });
            total_size += available_size;
        }

        ResolvedType::Message(Message {
            name: format_ident!("{}", self.name),
            ty: self.ty,
            version: self.version.parse().unwrap(),
            fields,
            size: total_size,
            chunk,
        })
    }

    pub fn get_reason(&self, table: &TypeTable) -> Result<()> {
        if table.contains(&self.name) {
            return Err(Error::AlreadyDefined(self.name.clone()));
        }

        if let Some(dup) = has_duplicates(
            &self
                .fields
                .iter()
                .map(|m| m.name.clone())
                .collect::<Vec<_>>(),
        ) {
            return Err(Error::DuplicateVariant {
                duplicate: dup.to_string(),
                src: self.name.clone(),
            });
        }

        for field in &self.fields {
            if table.get_type(&field.ty).is_none() {
                return Err(Error::MissingType(field.ty.clone()));
            }
        }

        if let Some(chunk) = self.chunk.as_ref() {
            return chunk.get_reason(table);
        }

        Ok(())
    }
}
