use std::ops::RangeInclusive;

use crate::{
    cpu::{MemoryPools, MessagePipeline, PipelineData},
    lookup::{LookupTable, SubscriberLookupData, SubscriberOldLookupData},
};

pub trait DataIterator: Iterator<Item = SubscriberLookupData> {}
pub trait OldDataIterator: Iterator<Item = SubscriberOldLookupData> {}

impl<I: Iterator<Item = SubscriberLookupData>> DataIterator for I {}
impl<I: Iterator<Item = SubscriberOldLookupData>> OldDataIterator for I {}

pub struct DriverAnswer<I1, I2> {
    pub subscriber_changes: I1,
    pub driver_changes: I2,
}

pub trait BusDriver {
    fn on_subscribe(&mut self, id: u8) -> DriverAnswer<impl DataIterator, impl DataIterator>;
    fn on_unsubscribe(
        &mut self,
        id: u8,
    ) -> DriverAnswer<impl OldDataIterator, impl OldDataIterator>;
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
        let answer = self.inner.on_subscribe(id);
        self.lookup_table.add(id, answer.subscriber_changes);
        //self.lookup_table.add(0, answer.driver_changes)
    }

    pub fn on_unsubscribe(&mut self, id: u8) {
        let answer = self.inner.on_unsubscribe(id);
        self.lookup_table.remove(id, answer.subscriber_changes);
        //self.lookup_table.remove(0, answer.driver_changes);
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
