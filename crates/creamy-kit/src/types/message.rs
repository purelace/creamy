use crate::{intern::StringId, types::Field};

#[derive(Debug, Clone)]
pub struct Message {
    name: StringId,
    fields: Vec<Field>,
}
