use std::num::NonZeroU8;

use cbus::{BusDriver, DataIterator, SubscriberLookupData, define_bus};
use cbus_core::{Subscriber, SubscriberId, UntypedMessage};

#[derive(Debug)]
struct Driver;
impl BusDriver for Driver {
    fn on_subscribe(&mut self, _: SubscriberId) -> impl DataIterator {
        core::iter::once(SubscriberLookupData {
            consumer_group_id: 10,
            provider_group_id: 1,
            provider_id: SubscriberId::new(NonZeroU8::new(1).unwrap()),
        })
    }

    fn on_unsubscribe(&mut self, _: SubscriberId) {}
}

define_bus! {
    max_messages: 2048,
    max_groups: 128,
    max_subscribers: 128,
}

struct Fuzzer {
    inc: IncBuf,
    out: OutBuf,
}

impl Subscriber for Fuzzer {
    fn notify(&mut self) {
        self.inc.clear();
        assert!(self.out.is_empty());
    }
}

struct Reader {
    inc: IncBuf,
    _out: OutBuf,
}

impl Subscriber for Reader {
    fn notify(&mut self) {
        self.inc.clear();
    }
}

type Bus = MessageBus<Driver>;

fn init() -> (Bus, OutBuf) {
    let mut bus = MessageBus::new(Driver);
    let inc = IncBuf::default();
    let out = OutBuf::default();
    bus.add_subscriber_with(
        inc.clone(),
        out.clone(),
        Box::new(Fuzzer {
            inc,
            out: out.clone(),
        }) as Box<dyn Subscriber>,
    )
    .unwrap();

    bus.add_subscriber(|inc, out| Box::new(Reader { inc, _out: out }) as Box<dyn Subscriber>)
        .unwrap();

    (bus, out)
}

#[test]
fn dst_is_more_than_max_possible() {
    let (mut bus, mut out) = init();
    assert!(out.send_many_iter_exact([
        UntypedMessage {
            dst: 238,
            group: 1,
            src: 17,
            kind: 0,
            payload: [0; 28],
        },
        UntypedMessage {
            dst: 128,
            group: 1,
            src: 0,
            kind: 0,
            payload: [0; 28],
        },
    ]));
    bus.full_tick();
}

#[test]
fn inactive_buffer_overflow() {
    let (mut bus, mut out) = init();

    for _ in 0..3 {
        let messages = core::iter::repeat_n(
            UntypedMessage {
                dst: 100, // несуществующий подписчких будет хранить данные без очистки.
                group: 1,
                src: 1,
                kind: 0,
                payload: [0; 28],
            },
            1024,
        )
        .collect::<Vec<_>>();
        for message in messages {
            out.send(message);
        }
        bus.full_tick();
    }
}

struct Sender {
    inc: IncBuf,
    _out: OutBuf,
}

impl Subscriber for Sender {
    fn notify(&mut self) {
        assert!(self.inc.count() == 0, "{:#?}", { self.inc.pop_all() });
    }
}

#[test]
fn self_send() {
    let mut bus = Bus::new(Driver);
    let inc = IncBuf::default();
    let mut out = OutBuf::default();
    bus.add_subscriber_with(
        inc.clone(),
        out.clone(),
        Box::new(Sender {
            inc,
            _out: out.clone(),
        }) as Box<dyn Subscriber>,
    )
    .unwrap();

    out.send_many_iter_exact(core::iter::repeat_n(
        UntypedMessage {
            dst: 1,
            group: 10,
            src: 1,
            kind: 111,
            payload: [0; 28],
        },
        1020,
    ));
    bus.full_tick();
}
