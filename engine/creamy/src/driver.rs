use creamy_engine_core::bus::{
    BusDriver, DataIterator, OldDataIterator, SubscriberLookupData, SubscriberOldLookupData,
};

pub struct EngineBusDriver {
    max_groups: u8,
    provide_requests: Vec<Vec<SubscriberLookupData>>,
    remove_requests: Vec<Vec<SubscriberOldLookupData>>,
}

impl EngineBusDriver {
    pub fn new(max_plugins: u8, max_groups: u8) -> Self {
        Self {
            max_groups,
            provide_requests: vec![vec![]; max_plugins as usize],
            remove_requests: vec![vec![]; max_plugins as usize],
        }
    }

    pub fn provide_api(&mut self, plugin: u8, request: SubscriberLookupData) {
        self.provide_requests[plugin as usize].push(request);
    }

    pub fn remove_api(&mut self, plugin: u8, request: SubscriberOldLookupData) {
        self.remove_requests[plugin as usize].push(request);
    }
}

impl BusDriver for EngineBusDriver {
    fn on_subscribe(&mut self, id: u8) -> impl DataIterator {
        self.provide_requests[id as usize].drain(..)
    }

    fn on_unsubscribe(&mut self, id: u8) -> impl OldDataIterator {
        self.remove_requests[id as usize].drain(..)
    }
}
