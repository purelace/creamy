use crate::{intern::StringId, types::Field};

#[derive(Debug, Clone)]
pub struct Structure {
    name: StringId,
    fields: Vec<Field>,
}
