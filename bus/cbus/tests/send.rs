#![allow(clippy::missing_errors_doc)]
#![allow(clippy::many_single_char_names)]

use cbus::{
    BusDriver, DataIterator, SubscriberLookupData,
    core::{
        Subscriber, SubscriberId, UntypedMessage,
        buffer::{IncBuf, OutBuf},
        subscribers,
    },
    define_bus_config,
};

const MAX_MESSAGES: usize = 11000;
define_bus_config! {
    Advanced,
    max_subscribers: u8::MAX - 1,
    max_messages: 11000,
    max_groups: 128,
}

#[derive(Debug)]
struct Driver;
impl BusDriver for Driver {
    fn on_subscribe(&mut self, id: SubscriberId) -> impl DataIterator {
        match id.get() {
            1 => std::iter::once(SubscriberLookupData {
                consumer_group_id: 0,
                provider_group_id: 0,
                provider_id: id,
            }),
            2 => std::iter::once(SubscriberLookupData {
                consumer_group_id: 10,
                provider_group_id: 1,
                provider_id:
                //: SAFETY 1 != 0
                unsafe {
                    SubscriberId::new_unchecked(1)
                },
            }),
            _ => unreachable!(),
        }
    }

    fn on_unsubscribe(&mut self, _: SubscriberId) {}
}

type MessageBus<S> = cbus::MessageBus<Advanced, Driver, MAX_MESSAGES, S>;

#[derive(Debug)]
struct TestListener<const A: usize> {
    incoming: IncBuf<MAX_MESSAGES>,
    _outgoing: OutBuf<MAX_MESSAGES>,
    total_messages: usize,
}

impl<const A: usize> TestListener<A> {
    pub const fn new(incoming: IncBuf<MAX_MESSAGES>, outgoing: OutBuf<MAX_MESSAGES>) -> Self {
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
            assert_eq!(message.group, 10);
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

#[derive(Debug)]
struct TestSender<const A: usize> {
    _incoming: IncBuf<MAX_MESSAGES>,
    outgoing: OutBuf<MAX_MESSAGES>,
}

impl<const A: usize> TestSender<A> {
    pub const fn new(incoming: IncBuf<MAX_MESSAGES>, outgoing: OutBuf<MAX_MESSAGES>) -> Self {
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

subscribers! {
    TSubs10000,
    Sender => TestSender::<10000>,
    Listener => TestListener::<10000>,
}

#[test]
pub fn send_1_message() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs1>::new(Driver);
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_10_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs10>::new(Driver);
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_100_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs100>::new(Driver);
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_1k_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs1000>::new(Driver);
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

#[test]
pub fn send_10k_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<TSubs10000>::new(Driver);
    bus.add_subscriber(TestSender::new)?;
    bus.add_subscriber(TestListener::new)?;
    bus.tick();
    bus.tick();

    Ok(())
}

/*
 * При регистрации мы должны правильно проставить значения в таблице прав.
 * Когда плагин уходит мы одну часть обнуляем полностью, а вторую должны изменить точечно.
 * Что исправить:
 * Buffer overflow
 */
