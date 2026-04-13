use std::ops::RangeInclusive;

use crate::{
    cpu::{MemoryPools, MessagePipeline, PipelineData},
    lookup::{LookupTable, SubscriberLookupData, SubscriberOldLookupData},
};

pub trait DataIterator: Iterator<Item = SubscriberLookupData> {}
pub trait OldDataIterator: Iterator<Item = SubscriberOldLookupData> {}

impl<I: Iterator<Item = SubscriberLookupData>> DataIterator for I {}
impl<I: Iterator<Item = SubscriberOldLookupData>> OldDataIterator for I {}

pub trait BusDriver {
    fn on_subscribe(&mut self, id: u8) -> impl DataIterator;
    fn on_unsubscribe(&mut self, id: u8) -> impl OldDataIterator;
}
pub struct Driver<D: BusDriver> {
    lookup_table: LookupTable,
    pipeline: MessagePipeline,
    inner: D,
}

impl<D: BusDriver> Driver<D> {
    pub fn new(driver: D, max_groups: u8, max_subscribers: u8) -> Self {
        Self {
            lookup_table: LookupTable::new(max_groups, max_subscribers),
            pipeline: MessagePipeline::new(max_subscribers),
            inner: driver,
        }
    }

    pub fn on_subscribe(&mut self, id: u8) {
        let iter = self.inner.on_subscribe(id);
        self.lookup_table.add(id, iter);
    }

    pub fn on_unsubscribe(&mut self, id: u8) {
        let iter = self.inner.on_unsubscribe(id);
        self.lookup_table.remove(id, iter);
    }

    pub fn process_messages(&mut self, memory: MemoryPools, range: RangeInclusive<usize>) {
        let mut data = PipelineData {
            lookup_table: &self.lookup_table,
            memory,
            subscriber_range: range,
        };

        self.pipeline.dispatch_messages(&mut data);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<D: BusDriver> Driver<D> {
    pub const fn get_inner(&self) -> &D {
        &self.inner
    }

    pub const fn get_inner_mut(&mut self) -> &mut D {
        &mut self.inner
    }
}
