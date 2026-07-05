#![allow(clippy::inline_always)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use cbus::{
    BusError, MessageBus,
    config::{BusConfig, Legacy, ValidConfig},
    core::{
        Subscriber,
        buffer::{Incoming, Outgoing},
    },
    define_bus_config,
    defines::DEFAULT_SLICE_SIZE,
};
use creamy_cbus_driver::CreamyDriver;

pub trait SenderFn: FnMut(&[u8], &mut Outgoing) + Send + Sync + 'static {}
impl<T> SenderFn for T where T: FnMut(&[u8], &mut Outgoing) + Send + Sync + 'static {}

pub enum Subscribers<F: SenderFn> {
    Sender(BenchmarkSender<F>),
    Listener(SimpleSubscriber),
}

impl<F: SenderFn> From<BenchmarkSender<F>> for Subscribers<F> {
    fn from(value: BenchmarkSender<F>) -> Self {
        Self::Sender(value)
    }
}

impl<F: SenderFn> From<SimpleSubscriber> for Subscribers<F> {
    fn from(value: SimpleSubscriber) -> Self {
        Self::Listener(value)
    }
}

impl<F: SenderFn> Subscriber for Subscribers<F> {
    fn notify(&mut self) {
        match self {
            Subscribers::Sender(value) => value.notify(),
            Subscribers::Listener(value) => value.notify(),
        }
    }
}

pub struct BenchmarkSender<S: SenderFn> {
    outgoing: Outgoing,
    _incoming: Incoming,
    vec: Vec<u8>,
    function: S,
}

impl<S: SenderFn> BenchmarkSender<S> {
    pub const fn new(outgoing: Outgoing, incoming: Incoming, vec: Vec<u8>, function: S) -> Self {
        Self {
            outgoing,
            _incoming: incoming,
            vec,
            function,
        }
    }
}

impl<S: SenderFn> Subscriber for BenchmarkSender<S> {
    fn notify(&mut self) {
        (self.function)(&self.vec, &mut self.outgoing);
    }
}

pub struct SimpleSubscriber {
    incoming: Incoming,
    _outgoing: Outgoing,
}

impl SimpleSubscriber {
    #[must_use]
    pub const fn new(incoming: Incoming, outgoing: Outgoing) -> Self {
        Self {
            incoming,
            _outgoing: outgoing,
        }
    }
}

impl Subscriber for SimpleSubscriber {
    fn notify(&mut self) {
        let messages = self.incoming.pop_all();
        for msg in messages {
            let val =
                unsafe { std::ptr::read_unaligned(std::ptr::from_ref(msg).cast::<[u64; 4]>()) };
            std::hint::black_box(val);
        }
    }
}

pub type BenchBus<F> = MessageBus<CreamyDriver, Subscribers<F>>;

pub fn init_bus_custom<C: BusConfig, F: SenderFn>(
    indices: Vec<u8>,
    function: F,
    subs: usize,
    config: C,
) -> Result<BenchBus<F>, Box<dyn std::error::Error>> {
    let mut bus = MessageBus::new(config, CreamyDriver::new)?;
    let sender_id =
        bus.add_subscriber(|inc, out| BenchmarkSender::new(out, inc, indices, function))?;

    let driver = bus.get_driver_mut();
    driver.declare_protocols("benchmark");
    driver.sync_protocol_table(sender_id.u8(), vec!["benchmark".to_string()]);

    for _ in 2..2 + subs {
        let listener_id = bus.add_subscriber(SimpleSubscriber::new).unwrap();
        bus.get_driver_mut()
            .sync_protocol_table(listener_id.u8(), vec!["benchmark".to_string()]);
    }

    bus.tick();
    Ok(bus)
}

pub fn init_bus_legacy<F: SenderFn>(
    indices: Vec<u8>,
    function: F,
    subs: usize,
) -> Result<BenchBus<F>, Box<dyn std::error::Error>> {
    init_bus_custom(indices, function, subs, Legacy)
}

define_bus_config! {
    LegacyLargeBuffer,
    max_subscribers: 32,
    max_messages: DEFAULT_SLICE_SIZE,
    max_groups: 128,
}

pub fn init_bus_legacy_large_buf<F: SenderFn>(
    indices: Vec<u8>,
    function: F,
    subs: usize,
) -> Result<BenchBus<F>, Box<dyn std::error::Error>> {
    init_bus_custom(indices, function, subs, LegacyLargeBuffer)
}

define_bus_config! {
    LegacyUltraLargeBuffer,
    max_subscribers: 32,
    max_messages: DEFAULT_SLICE_SIZE * 16,
    max_groups: 128,
}

pub fn init_bus_legacy_ularge_buf<F: SenderFn>(
    indices: Vec<u8>,
    function: F,
    subs: usize,
) -> Result<BenchBus<F>, Box<dyn std::error::Error>> {
    init_bus_custom(indices, function, subs, LegacyUltraLargeBuffer)
}
