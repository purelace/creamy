use alloc::{boxed::Box, vec, vec::Vec};

use creamy_engine_core::bus::{BusDriver, DataIterator, SubscriberLookupData};
use creamy_sdk::SubscriberId;

pub struct EngineBusDriver<const S: usize> {
    requests: Box<[Vec<SubscriberLookupData>; S]>,
}

impl<const S: usize> EngineBusDriver<S> {
    pub fn new() -> Self {
        Self {
            requests: Box::new(core::array::from_fn(|_| vec![])),
        }
    }

    pub fn provide_api(&mut self, id: SubscriberId, request: SubscriberLookupData) {
        self.requests[id.get() as usize - 1].push(request);
    }
}

impl<const S: usize> BusDriver for EngineBusDriver<S> {
    fn on_subscribe(&mut self, id: SubscriberId) -> impl DataIterator {
        self.requests[id.get() as usize - 1].drain(..)
    }

    fn on_unsubscribe(&mut self, id: SubscriberId) {
        self.requests[id.get() as usize - 1].clear();
    }
}
