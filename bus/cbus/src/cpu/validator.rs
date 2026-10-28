use crate::{
    config::BusConfig,
    lookup::LookupTable,
    sys::{SubscriberRange, subscriber::Mask},
};

#[derive(Clone, Copy)]
pub struct Validator<'a, C: BusConfig> {
    pub lut: &'a LookupTable<C>,
    pub range: SubscriberRange,
    pub mask: Mask,
    pub src: u8,
}

impl<C: BusConfig> Validator<'_, C> {
    fn exist(&self, dst: u8) -> u8 {
        let in_valid_range = u8::from(self.range.contains(dst));
        let exist = self.mask.get_bit(dst);
        in_valid_range * exist
    }

    fn valid_group(&self, group: u8) -> u8 {
        self.lut.transform_group(group, self.src)
    }

    pub fn validate_dst(&self, dst: u8, group: u8) -> u8 {
        let not_equal_src = u8::from(self.src != dst);
        dst * not_equal_src * self.exist(dst) * self.valid_group(group)
    }
}
