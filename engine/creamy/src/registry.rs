use std::collections::HashMap;

use creamy_engine_core::devkit::{
    utils::strpool::StringPool,
    xmlc::{ProtocolDefinition, StringPoolResolver},
};

#[derive(Default)]
pub struct ProtocolRegistry {
    pool: StringPool,
    map: HashMap<Box<str>, ProtocolDefinition>,
}

impl ProtocolRegistry {
    pub(crate) fn replace_strings(
        &mut self,
        pool: &StringPool,
        definition: &mut ProtocolDefinition,
    ) {
        definition.replace_identifiers(pool, &mut self.pool);
    }

    pub(crate) fn declare_protocol(
        &mut self,
        name: impl Into<Box<str>>,
        definition: ProtocolDefinition,
    ) {
        let path = name.into();
        tracing::info!(
            "Protocol '{}@{}' declared",
            definition.name().resolve(&self.pool),
            definition.version()
        );
        //TODO: chech if contains
        assert!(self.map.insert(path, definition).is_none());
    }

    pub fn get_model(&self, name: &str) -> Option<&ProtocolDefinition> {
        self.map.get(name)
    }

    pub const fn pool(&self) -> &StringPool {
        &self.pool
    }
}
