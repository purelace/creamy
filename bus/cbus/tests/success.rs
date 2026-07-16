#![allow(clippy::missing_errors_doc)]
#![allow(clippy::many_single_char_names)]

use cbus::{
    BusDriver, DataIterator, OldDataIterator, SubscriberLookupData, SubscriberOldLookupData,
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
    fn on_subscribe(&mut self, id: u8) -> impl DataIterator {
        match id {
            1 | 2 => std::iter::once(SubscriberLookupData {
                local_group_id: 1,
                global_group_id: 10,
            }),
            _ => unreachable!(),
        }
    }

    fn on_unsubscribe(&mut self, _id: u8) -> impl OldDataIterator {
        std::iter::once(SubscriberOldLookupData {
            global_group_id: 10,
        })
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
        while let Some(message) = self.incoming.pop() {
            assert_eq!(message.dst, 2);
            assert_eq!(message.src, 1);
            assert_eq!(message.group, 1);
            assert_eq!(message.kind, 1);

            let [a, b, c, d] = message.payload[0..4].try_into().unwrap();
            let zeros: [u8; 20] = message.payload[4..24].try_into().unwrap();
            let [e, f, g, h] = message.payload[24..28].try_into().unwrap();

            assert_eq!(zeros, [0; 20]);
            assert_eq!(
                usize::from_le_bytes([a, b, c, d, e, f, g, h]),
                self.total_messages
            );

            self.total_messages += 1;
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
            payload: {
                let [a, b, c, d, e, f, g, h] = i.to_le_bytes();
                [
                    a, b, c, d, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0, 0, //
                    e, f, g, h, //
                ]
            },
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
    let mut bus = MessageBus::<TSubs1>::new(&Advanced.into_valid()?, Driver::new);
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_10_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs10>::new(&Advanced.into_valid()?, Driver::new);
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_100_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs100>::new(&Advanced.into_valid()?, Driver::new);
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_1k_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs1000>::new(&Advanced.into_valid()?, Driver::new);
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}
