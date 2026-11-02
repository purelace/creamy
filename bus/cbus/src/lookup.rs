use alloc::{boxed::Box, vec};

use cbus_core::SubscriberId;

#[derive(Debug, Clone, Copy)]
pub struct SubscriberLookupData {
    pub consumer_group_id: u8,
    pub provider_group_id: u8,
    pub provider_id: SubscriberId,
}

#[derive(Debug)]
pub struct LookupTable<const MG: usize, const MS: usize> {
    input: Box<[u8]>,
}

impl<const MG: usize, const MS: usize> LookupTable<MG, MS> {
    pub fn new() -> Self {
        Self {
            input: vec![0u8; MG * MS].into(),
        }
    }

    pub fn add<I>(&mut self, id: SubscriberId, lookup_data_iter: I)
    where
        I: IntoIterator<Item = SubscriberLookupData>,
    {
        let id = id.as_usize();

        for lookup_data in lookup_data_iter {
            self.set_values(id, lookup_data);
            self.set_values(
                lookup_data.provider_id.as_usize(),
                SubscriberLookupData {
                    consumer_group_id: lookup_data.provider_group_id,
                    provider_group_id: lookup_data.consumer_group_id,
                    provider_id: lookup_data.provider_id,
                },
            );
        }
    }

    fn set_values(&mut self, id: usize, data: SubscriberLookupData) {
        let slice = id * MG;
        let src_index = slice + data.consumer_group_id as usize;
        self.input[src_index] = data.provider_group_id;
    }

    pub fn zeroize(&mut self, id: SubscriberId) {
        let id = id.as_usize();

        let start = id * MG;
        let end = start + MG;

        let in_slice = &mut self.input[start..end];
        in_slice.fill(0);
    }

    pub const fn as_slice(&self) -> &[u8] {
        &self.input
    }
}
