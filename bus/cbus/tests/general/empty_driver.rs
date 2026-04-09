use cbus::{
    BusDriver, DataIterator, DriverAnswer, OldDataIterator,
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
    fn on_subscribe(&mut self, _id: u8) -> DriverAnswer<impl DataIterator, impl DataIterator> {
        DriverAnswer {
            subscriber_changes: std::iter::empty(),
            driver_changes: std::iter::empty(),
        }
    }

    fn on_unsubscribe(
        &mut self,
        _id: u8,
    ) -> DriverAnswer<impl OldDataIterator, impl OldDataIterator> {
        DriverAnswer {
            subscriber_changes: std::iter::empty(),
            driver_changes: std::iter::empty(),
        }
    }
}
