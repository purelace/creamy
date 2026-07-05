use std::collections::HashMap;

use cbus::{
    BusDriver, DataIterator, OldDataIterator, SubscriberLookupData, SubscriberOldLookupData,
    config::{BusConfig, ValidConfig},
    core::buffer::Outgoing,
};
use idmint::StackMint;

#[derive(Default, Clone)]
struct SubscriberMetadata {
    to_sync: Vec<String>,
    groups: Vec<u8>,
}

pub struct CreamyDriver {
    _outgoing: Outgoing,

    provider: StackMint,
    groups: HashMap<String, u8>,
    subscribers: Vec<SubscriberMetadata>,
}

impl CreamyDriver {
    #[must_use]
    pub fn new<C: BusConfig>(c: &ValidConfig<C>, outgoing: Outgoing) -> Self {
        let subscribers =
            std::iter::repeat_n(SubscriberMetadata::default(), c.max_subscribers() as usize)
                .collect::<Vec<_>>();

        Self {
            _outgoing: outgoing,
            groups: HashMap::new(),
            provider: StackMint::new(1),
            subscribers,
        }
    }

    pub fn declare_protocols(&mut self, name: impl Into<String>) {
        let id = self.provider.issue().unwrap();
        assert!(self.groups.insert(name.into(), id).is_none());
    }

    pub fn sync_protocol_table(&mut self, id: u8, protocols: Vec<String>) {
        let metadata = &mut self.subscribers[id as usize];
        metadata.to_sync.extend(protocols);
    }
}

impl BusDriver for CreamyDriver {
    fn on_subscribe(&mut self, id: u8) -> impl DataIterator {
        let metadata = &mut self.subscribers[id as usize];

        metadata
            .to_sync
            .drain(..)
            .enumerate()
            .map(|(local_group_id, name)| {
                let local_group_id = local_group_id + 1;
                let global_group_id = *self.groups.get(&name).unwrap();

                metadata.groups.push(global_group_id);

                SubscriberLookupData {
                    local_group_id,
                    global_group_id,
                }
            })
    }

    fn on_unsubscribe(&mut self, id: u8) -> impl OldDataIterator {
        let metadata = &mut self.subscribers[id as usize];
        metadata.to_sync.clear();

        metadata
            .groups
            .drain(..)
            .map(|global_group_id| SubscriberOldLookupData { global_group_id })
    }
}
