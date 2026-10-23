#![no_main]

use std::sync::{Mutex, OnceLock};

use cbus::{BusDriver, DataIterator, MessageBus, SubscriberLookupData, define_bus_config};
use cbus_core::{
    Subscriber, SubscriberId, UntypedMessage,
    buffer::{IncBuf, OutBuf},
};
use libfuzzer_sys::fuzz_target;

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

const M: usize = 2048;
define_bus_config! {
    Config,
    max_subscribers: 128,
    max_messages: 2048,
    max_groups: 128,
}

struct Wrapper {
    bus: MessageBus<Config, Driver, M>,
    out: OutBuf<M>,
}

unsafe impl Send for Wrapper {}
unsafe impl Sync for Wrapper {}

static BUS: OnceLock<Mutex<Wrapper>> = OnceLock::new();

struct Fuzzer {
    inc: IncBuf<M>,
    _out: OutBuf<M>,
}

impl Subscriber for Fuzzer {
    fn notify(&mut self) {
        assert!(self.inc.count() == 0, "{:#?}", { self.inc.pop_all() });
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

fn init() -> Mutex<Wrapper> {
    let mut bus = MessageBus::new(Driver);
    let inc = IncBuf::default();
    let out = OutBuf::default();
    bus.add_subscriber_with(
        inc.clone(),
        out.clone(),
        Box::new(Fuzzer {
            inc,
            _out: out.clone(),
        }) as Box<dyn Subscriber>,
    )
    .unwrap();

    bus.add_subscriber(|inc, out| Box::new(Reader { inc, _out: out }) as Box<dyn Subscriber>)
        .unwrap();

    Mutex::new(Wrapper { bus, out })
}

fuzz_target!(|messages: Vec<UntypedMessage>| {
    let mutex = BUS.get_or_init(init);
    let data = &mut mutex.lock().unwrap();
    data.out.send_many_iter_exact(messages.into_iter());
    data.bus.full_tick();
});
