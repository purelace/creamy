use std::collections::HashMap;

use creamy_engine_core::devkit::{
    utils::strpool::StringPool,
    xmlc::{ProtocolDefinition, StringPoolResolver},
};

#[derive(Clone)]
pub enum AccessPolicy {
    Public { consumers: Box<[u8]> },
    Private { consumer: Option<u8> },
}

impl Default for AccessPolicy {
    fn default() -> Self {
        Self::Private { consumer: None }
    }
}

pub struct ProtocolRuntimeData {
    definition: ProtocolDefinition,
    storage: Box<[AccessPolicy]>,
    owner: u8,
}

impl ProtocolRuntimeData {
    fn new(definition: ProtocolDefinition, owner: u8) -> Self {
        let groups = definition.groups().len();
        let storage = std::iter::repeat_n(AccessPolicy::default(), groups).collect();
        Self {
            definition,
            storage,
            owner,
        }
    }
}

#[derive(Default)]
pub struct ProtocolRegistry {
    pool: StringPool,
    map: HashMap<Box<str>, ProtocolRuntimeData>,
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
        owner: u8,
    ) {
        let path = name.into();
        tracing::info!(
            "Protocol '{}@{}' declared",
            definition.name().resolve(&self.pool),
            definition.version()
        );

        let data = ProtocolRuntimeData::new(definition, owner);

        //TODO: chech if contains
        assert!(self.map.insert(path, data).is_none());
    }

    pub fn get_model(&self, name: &str) -> Option<&ProtocolDefinition> {
        self.map.get(name).map(|data| &data.definition)
    }

    pub const fn pool(&self) -> &StringPool {
        &self.pool
    }
}
