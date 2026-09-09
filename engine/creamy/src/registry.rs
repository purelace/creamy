use alloc::boxed::Box;
use core::num::NonZeroU8;

use creamy_engine_core::devkit::compiler::{
    ProtocolDefinition,
    utils::strpool::{StringId, StringPool, StringPoolResolver},
};
use creamy_sdk::bus::SubscriberId;
use rustc_hash::FxHashMap;

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
    owner: SubscriberId,
    table: FxHashMap<StringId, NonZeroU8>,
}

impl ProtocolRuntimeData {
    fn new(definition: ProtocolDefinition, owner: SubscriberId) -> Self {
        let groups = definition.groups().len();
        let storage = core::iter::repeat_n(AccessPolicy::default(), groups).collect();
        Self {
            definition,
            storage,
            owner,
            table: FxHashMap::default(),
        }
    }

    pub const fn definition(&self) -> &ProtocolDefinition {
        &self.definition
    }

    pub const fn owner(&self) -> SubscriberId {
        self.owner
    }

    pub fn get_provider_group_id(&self, group: StringId) -> Option<NonZeroU8> {
        self.table.get(&group).copied()
    }
}

#[derive(Default)]
pub struct ProtocolRegistry {
    pool: StringPool,
    map: FxHashMap<Box<str>, ProtocolRuntimeData>,
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
        owner: SubscriberId,
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

    pub fn get_protocol_context(&self, name: &str) -> Option<&ProtocolRuntimeData> {
        self.map.get(name).map(|data| data)
    }

    pub fn get_group(&self, protocol: &str, group: &str) -> Option<NonZeroU8> {
        let group_name = self.pool.get_id(group);
        self.map
            .get(protocol)
            .and_then(|data| data.table.get(&group_name))
            .copied()
    }

    pub(crate) fn set_group_id(&mut self, protocol: &str, group: &str, id: NonZeroU8) {
        let group_name = self.pool.get_id(group);

        if let Some(data) = self.map.get_mut(protocol) {
            data.table.insert(group_name, id);
        }
    }

    //pub(crate) fn remove_group(&mut self, protocol: &str, group)

    pub const fn pool(&self) -> &StringPool {
        &self.pool
    }
}
