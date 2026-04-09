#![allow(clippy::missing_errors_doc)]

use cbus::{
    BusDriver, DataIterator, DriverAnswer, OldDataIterator, SubscriberLookupData,
    SubscriberOldLookupData,
    config::{Advanced, BusConfig, ValidConfig},
    core::{
        Subscriber, UntypedMessage,
        buffer::{Incoming, Outgoing},
        subscribers,
    },
};

struct Driver;

impl Driver {
    pub fn new<C: BusConfig>(_config: &ValidConfig<C>, _outgoing: Outgoing) -> Self {
        Self
    }
}

impl BusDriver for Driver {
    fn on_subscribe(&mut self, id: u8) -> DriverAnswer<impl DataIterator, impl DataIterator> {
        match id {
            1 | 2 => DriverAnswer {
                subscriber_changes: std::iter::once(SubscriberLookupData {
                    local_group_id: 1,
                    global_group_id: 10,
                }),
                driver_changes: std::iter::empty(),
            },
            _ => unreachable!(),
        }
    }

    fn on_unsubscribe(
        &mut self,
        _id: u8,
    ) -> DriverAnswer<impl OldDataIterator, impl OldDataIterator> {
        DriverAnswer {
            subscriber_changes: std::iter::once(SubscriberOldLookupData {
                global_group_id: 10,
            }),
            driver_changes: std::iter::empty(),
        }
    }
}

type MessageBus<S> = cbus::MessageBus<Driver, S>;

struct TestListener<const A: usize> {
    incoming: Incoming,
    _outgoing: Outgoing,
    total_messages: usize,
}

impl<const A: usize> TestListener<A> {
    pub const fn new(incoming: Incoming, outgoing: Outgoing) -> Self {
        Self {
            incoming,
            _outgoing: outgoing,
            total_messages: 0,
        }
    }
}

impl<const A: usize> Subscriber for TestListener<A> {
    fn notify(&mut self) {
        self.total_messages += self.incoming.count() as usize;
        //let mut version = 0;
        while let Some(message) = self.incoming.pop() {
            assert_eq!(message.dst, 2);
            assert_eq!(message.src, 1);
            assert_eq!(message.group, 1);
            assert_eq!(message.kind, 1);
            //assert_eq!(message.version, version);
            //version += 1;
        }
    }
}

impl<const A: usize> Drop for TestListener<A> {
    fn drop(&mut self) {
        assert_eq!(self.total_messages, A);
    }
}

struct TestSender<const A: usize> {
    _incoming: Incoming,
    outgoing: Outgoing,
}

impl<const A: usize> TestSender<A> {
    pub const fn new(incoming: Incoming, outgoing: Outgoing) -> Self {
        Self {
            _incoming: incoming,
            outgoing,
        }
    }
}

impl<const A: usize> Subscriber for TestSender<A> {
    fn notify(&mut self) {
        let iter = (0..A).map(|i| UntypedMessage {
            dst: 2,
            group: 1,
            src: 0,
            kind: 1,
            version: 0,
            payload: [0; 27],
        });

        assert!(self.outgoing.send_many_iter_exact(iter));
    }
}

subscribers! {
    TSubs1,
    Sender => TestSender::<1>,
    Listener => TestListener::<1>,
}

subscribers! {
    TSubs10,
    Sender => TestSender::<10>,
    Listener => TestListener::<10>,
}

subscribers! {
    TSubs100,
    Sender => TestSender::<100>,
    Listener => TestListener::<100>,
}

subscribers! {
    TSubs1000,
    Sender => TestSender::<1000>,
    Listener => TestListener::<1000>,
}

#[test]
pub fn send_1_message() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs1>::new(Advanced, Driver::new)?;
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_10_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs10>::new(Advanced, Driver::new)?;
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_100_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs100>::new(Advanced, Driver::new)?;
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_1k_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs1000>::new(Advanced, Driver::new)?;
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}
