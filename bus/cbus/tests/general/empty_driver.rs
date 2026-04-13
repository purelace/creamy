use cbus::{
    BusDriver, DataIterator, OldDataIterator,
    config::{BusConfig, ValidConfig},
    core::buffer::Outgoing,
};

pub struct EmptyDriver;
impl EmptyDriver {
    pub fn new<C: BusConfig>(_config: &ValidConfig<C>, _outgoing: Outgoing) -> Self {
        Self
    }
}
impl BusDriver for EmptyDriver {
    fn on_subscribe(&mut self, _id: u8) -> impl DataIterator {
        std::iter::empty()
    }

    fn on_unsubscribe(&mut self, _id: u8) -> impl OldDataIterator {
        std::iter::empty()
    }
}
