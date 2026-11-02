use core::marker::PhantomData;

use crate::{
    core::{Subscriber, SubscriberId},
    cpu::{MemoryPools, MessagePipeline, PipelineData},
    lookup::{LookupTable, SubscriberLookupData},
    sys::SubscriberRange,
};

pub trait DataIterator: Iterator<Item = SubscriberLookupData> {}
impl<I: Iterator<Item = SubscriberLookupData>> DataIterator for I {}

pub trait BusDriver {
    fn on_subscribe(&mut self, id: SubscriberId) -> impl DataIterator;
    fn on_unsubscribe(&mut self, id: SubscriberId);
}

#[derive(Debug)]
pub struct Driver<D, S, const MM: usize, const MG: usize, const MS: usize>
where
    D: BusDriver,
    S: Subscriber,
{
    lookup_table: LookupTable<MG, MS>,
    pipeline: MessagePipeline<S, MM, MG, MS>,
    inner: D,
    _phantom: PhantomData<S>,
}

impl<D, S, const MM: usize, const MG: usize, const MS: usize> Driver<D, S, MM, MG, MS>
where
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

    pub fn process_messages(&mut self, memory: MemoryPools<S, MM, MS>, range: SubscriberRange) {
        let mut data = PipelineData {
            lookup_table: &self.lookup_table,
            memory,
            subscriber_range: range,
        };

        self.pipeline.dispatch_messages(&mut data);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<D, S, const MM: usize, const MG: usize, const MS: usize> Driver<D, S, MM, MG, MS>
where
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
