use std::num::NonZeroU8;

use cbus::{BusDriver, DataIterator, SubscriberLookupData, define_bus_config};
use cbus_core::{
    Subscriber, SubscriberId, UntypedMessage,
    buffer::{IncBuf, OutBuf},
};

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

const M: usize = 2048;
define_bus_config! {
    Config,
    max_subscribers: 128,
    max_messages: 2048,
    max_groups: 128,
}

struct Fuzzer {
    inc: IncBuf<M>,
    out: OutBuf<M>,
}

impl Subscriber for Fuzzer {
    fn notify(&mut self) {
        self.inc.clear();
        assert!(self.out.is_empty());
    }
}

struct Reader {
    inc: IncBuf<M>,
    _out: OutBuf<M>,
}

impl Subscriber for Reader {
    fn notify(&mut self) {
        self.inc.clear();
    }
}

type MessageBus = cbus::MessageBus<Config, Driver, M>;

fn init() -> (MessageBus, OutBuf<M>) {
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
