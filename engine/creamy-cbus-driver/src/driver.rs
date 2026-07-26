use std::collections::HashMap;

use cbus::{BusDriver, DataIterator, SubscriberLookupData, config::BusConfig, core::SubscriberId};
use idmint::StackMint;

#[derive(Default, Clone)]
struct SubscriberMetadata {
    to_sync: Vec<String>,
    groups: Vec<u8>,
}

pub struct CreamyDriver {
    provider: StackMint,
    groups: HashMap<String, (SubscriberId, u8)>,
    subscribers: Vec<SubscriberMetadata>,
}

impl CreamyDriver {
    #[must_use]
    pub fn new<C: BusConfig>() -> Self {
        let subscribers = core::iter::repeat_n(
            SubscriberMetadata::default(),
            C::MAX_SUBSCRIBERS.get() as usize,
        )
        .collect::<Vec<_>>();

        Self {
            groups: HashMap::new(),
            provider: StackMint::new(1),
            subscribers,
        }
    }

    pub fn declare_protocols(&mut self, name: impl Into<String>, provider: SubscriberId) {
        let id = self.provider.issue().unwrap();
        assert!(self.groups.insert(name.into(), (provider, id)).is_none());
    }

    pub fn sync_protocol_table(&mut self, id: u8, protocols: Vec<String>) {
        let metadata = &mut self.subscribers[id as usize];
        metadata.to_sync.extend(protocols);
    }
}

impl BusDriver for CreamyDriver {
    fn on_subscribe(&mut self, id: SubscriberId) -> impl DataIterator {
        let metadata = &mut self.subscribers[id.get() as usize];

        metadata
            .to_sync
            .drain(..)
            .enumerate()
            .map(|(consumer_group_id, name)| {
                let consumer_group_id = consumer_group_id as u8 + 1;
                let (provider_id, provider_group_id) = *self.groups.get(&name).unwrap();

                metadata.groups.push(provider_group_id);

                SubscriberLookupData {
                    consumer_group_id,
                    provider_group_id,
                    provider_id,
                }
            })
    }

    fn on_unsubscribe(&mut self, id: SubscriberId) {
        let metadata = &mut self.subscribers[id.get() as usize];
        metadata.to_sync.clear();
    }
}
