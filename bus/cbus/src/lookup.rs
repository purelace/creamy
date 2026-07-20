use alloc::{vec, vec::Vec};

#[derive(Debug, Clone, Copy)]
pub struct SubscriberLookupData {
    pub consumer_group_id: u8,
    pub provider_group_id: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct SubscriberOldLookupData {
    pub provider_group_id: u8,
}

pub struct LookupTable {
    input: Vec<u8>,
    output: Vec<u8>,
    max_groups: u8,
}

impl LookupTable {
    pub fn new(max_groups_u8: u8, max_subscribers: u8) -> Self {
        let max_subscribers = max_subscribers as usize;
        let max_groups = max_groups_u8 as usize;

        Self {
            input: vec![0u8; max_subscribers * max_groups],
            output: vec![0u8; max_subscribers * max_groups],
            max_groups: max_groups_u8,
        }
    }

    pub fn add<I>(&mut self, id: u8, lookup_data_iter: I)
    where
        I: IntoIterator<Item = SubscriberLookupData>,
    {
        let id = id as usize;

        for lookup_data in lookup_data_iter {
            // Индекс в IN: (ID подписчика * max groups) + Локальный ID группы
            let in_idx = (id * self.max_groups as usize) + lookup_data.consumer_group_id as usize;
            self.input[in_idx] = lookup_data.provider_group_id;

            // Индекс в OUT: (ID подписчика * max groups) + Глобальный ID группы
            let out_idx = (id * self.max_groups as usize) + lookup_data.provider_group_id as usize;
            self.output[out_idx] = lookup_data.consumer_group_id;

            self.input[id * self.max_groups as usize] = 1;
            self.output[id * self.max_groups as usize] = 1;
        }
    }

    pub fn remove<I>(&mut self, id: u8, lookup_data_iter: I)
    where
        I: IntoIterator<Item = SubscriberOldLookupData>,
    {
        let id = id as usize;
        let max_groups = self.max_groups as usize;

        for lookup_data in lookup_data_iter {
            let out_idx = (id * max_groups) + lookup_data.provider_group_id as usize;
            let local_group_id = self.output[out_idx];

            let in_idx = (id * max_groups) + local_group_id as usize;
            self.input[in_idx] = 0;
            self.output[out_idx] = 0;

            self.input[id * self.max_groups as usize] = 0;
            self.output[id * self.max_groups as usize] = 0;
        }
    }

    pub const fn get_input(&self) -> &[u8] {
        self.input.as_slice()
    }

    pub const fn get_output(&self) -> &[u8] {
        self.output.as_slice()
    }

    pub const fn max_groups(&self) -> usize {
        self.max_groups as usize
    }
}
