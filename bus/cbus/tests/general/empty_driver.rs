use std::num::NonZeroU8;

use cbus::{BusDriver, DataIterator, config::BusConfig, core::SubscriberId};

#[derive(Debug)]
pub struct EmptyDriver;
impl BusDriver for EmptyDriver {
    fn on_subscribe(&mut self, _: SubscriberId) -> impl DataIterator {
        std::iter::empty()
    }

    fn on_unsubscribe(&mut self, _: SubscriberId) {}
}
