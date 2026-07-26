use core::{marker::PhantomData, ops::RangeInclusive};

use crate::{
    config::BusConfig,
    core::{Subscriber, SubscriberId},
    cpu::{MemoryPools, MessagePipeline, PipelineData},
    lookup::{LookupTable, SubscriberLookupData},
};

pub trait DataIterator: Iterator<Item = SubscriberLookupData> {}
impl<I: Iterator<Item = SubscriberLookupData>> DataIterator for I {}

pub trait BusDriver {
    fn on_subscribe(&mut self, id: SubscriberId) -> impl DataIterator;
    fn on_unsubscribe(&mut self, id: SubscriberId);
}

#[derive(Debug)]
pub struct Driver<C, D, S, const M: usize>
where
    C: BusConfig,
    D: BusDriver,
    S: Subscriber,
{
    lookup_table: LookupTable<C>,
    pipeline: MessagePipeline<C, S, M>,
    inner: D,
    _phantom: PhantomData<(C, S)>,
}

impl<C, D, S, const M: usize> Driver<C, D, S, M>
where
    C: BusConfig,
    D: BusDriver,
    S: Subscriber,
{
    pub fn new(driver: D) -> Self {
        Self {
            lookup_table: LookupTable::new(),
            pipeline: MessagePipeline::new(),
            inner: driver,
            _phantom: PhantomData,
        }
    }

    pub fn on_subscribe(&mut self, id: SubscriberId) {
        let iter = self.inner.on_subscribe(id);
        self.lookup_table.add(id, iter);
    }

    pub fn on_unsubscribe(&mut self, id: SubscriberId) {
        self.inner.on_unsubscribe(id);
        self.lookup_table.zeroize(id);
    }

    pub fn process_messages(&mut self, memory: MemoryPools<C, S, M>, range: RangeInclusive<u8>) {
        let mut data = PipelineData {
            lookup_table: &self.lookup_table,
            memory,
            subscriber_range: range,
            _phantom: PhantomData,
        };

        self.pipeline.dispatch_messages(&mut data);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<C, D, S, const M: usize> Driver<C, D, S, M>
where
    C: BusConfig,
    D: BusDriver,
    S: Subscriber,
{
    pub const fn get_inner(&self) -> &D {
        &self.inner
    }

    pub const fn get_inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }
}
