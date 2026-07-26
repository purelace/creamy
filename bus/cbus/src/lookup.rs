use alloc::{boxed::Box, vec};
use core::marker::PhantomData;

use cbus_core::SubscriberId;

use crate::config::BusConfig;

#[derive(Debug, Clone, Copy)]
pub struct SubscriberLookupData {
    pub consumer_group_id: u8,
    pub provider_group_id: u8,
    pub provider_id: SubscriberId,
}

#[derive(Debug)]
pub struct LookupTable<C: BusConfig> {
    input: Box<[u8]>,
    _phantom: PhantomData<C>,
}

impl<C: BusConfig> LookupTable<C> {
    pub const MAX_GROUPS: usize = C::MAX_GROUPS.get() as usize;

    pub fn new() -> Self {
        let max_subscribers = C::MAX_SUBSCRIBERS.get() as usize;

        Self {
            input: vec![0u8; max_subscribers * Self::MAX_GROUPS].into(),
            //output: vec![0u8; max_subscribers * Self::MAX_GROUPS].into(),
            _phantom: PhantomData,
        }
    }

    pub fn add<I>(&mut self, id: SubscriberId, lookup_data_iter: I)
    where
        I: IntoIterator<Item = SubscriberLookupData>,
    {
        let id = id.get() as usize;

        for lookup_data in lookup_data_iter {
            self.set_values(id, lookup_data);
            self.set_values(
                lookup_data.provider_id.get() as usize,
                SubscriberLookupData {
                    consumer_group_id: lookup_data.provider_group_id,
                    provider_group_id: lookup_data.consumer_group_id,
                    provider_id: lookup_data.provider_id,
                },
            );

            //self.input[id * Self::MAX_GROUPS] = 1;
            //self.output[id * Self::MAX_GROUPS] = 1;
        }
    }

    fn set_values(&mut self, id: usize, data: SubscriberLookupData) {
        let slice = id * Self::MAX_GROUPS;
        // Индекс в IN: (ID подписчика * max groups) + ID группы потребителя
        let in_idx = slice + data.consumer_group_id as usize;
        self.input[in_idx] = data.provider_group_id;

        // Индекс в OUT: (ID подписчика * max groups) + ID группы поставщика
        //let out_idx = slice + data.provider_group_id as usize;
        //self.output[out_idx] = data.consumer_group_id;
    }

    pub fn zeroize(&mut self, id: SubscriberId) {
        let id = id.get() as usize;

        let start = id * Self::MAX_GROUPS;
        let end = start + Self::MAX_GROUPS;

        let in_slice = &mut self.input[start..end];
        for value in in_slice {
            *value = 0;
        }

        //let out_slice = &mut self.output[start..end];
        //for value in out_slice {
        //    *value = 0;
        //}
    }

    pub const fn get_input(&self) -> &[u8] {
        &self.input
    }

    //pub const fn get_output(&self) -> &[u8] {
    //    &self.output
    //}
}
