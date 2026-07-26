#![allow(clippy::inline_always)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use cbus::{
    MessageBus,
    config::BusConfig,
    core::{
        Subscriber,
        buffer::{IncBuf, OutBuf},
    },
    define_bus_config,
};
use creamy_cbus_driver::CreamyDriver;

pub trait SenderFn<const M: usize>: FnMut(&[u8], &mut OutBuf<M>) + Send + Sync + 'static {}
impl<T, const M: usize> SenderFn<M> for T where
    T: FnMut(&[u8], &mut OutBuf<M>) + Send + Sync + 'static
{
}

pub enum Subscribers<F: SenderFn<M>, const M: usize> {
    Sender(BenchmarkSender<F, M>),
    Listener(SimpleSubscriber<M>),
}

impl<F: SenderFn<M>, const M: usize> From<BenchmarkSender<F, M>> for Subscribers<F, M> {
    fn from(value: BenchmarkSender<F, M>) -> Self {
        Self::Sender(value)
    }
}

impl<F: SenderFn<M>, const M: usize> From<SimpleSubscriber<M>> for Subscribers<F, M> {
    fn from(value: SimpleSubscriber<M>) -> Self {
        Self::Listener(value)
    }
}

impl<F: SenderFn<M>, const M: usize> Subscriber for Subscribers<F, M> {
    fn notify(&mut self) {
        match self {
            Subscribers::Sender(value) => value.notify(),
            Subscribers::Listener(value) => value.notify(),
        }
    }
}

pub struct BenchmarkSender<S: SenderFn<M>, const M: usize> {
    outgoing: OutBuf<M>,
    _incoming: IncBuf<M>,
    vec: Vec<u8>,
    function: S,
    work_finished: bool,
}

impl<S: SenderFn<M>, const M: usize> BenchmarkSender<S, M> {
    pub const fn new(outgoing: OutBuf<M>, incoming: IncBuf<M>, vec: Vec<u8>, function: S) -> Self {
        Self {
            outgoing,
            _incoming: incoming,
            vec,
            function,
            work_finished: false,
        }
    }
}

impl<S: SenderFn<M>, const M: usize> Subscriber for BenchmarkSender<S, M> {
    fn notify(&mut self) {
        if self.work_finished {
            return;
        }

        self.work_finished = true;
        (self.function)(&self.vec, &mut self.outgoing);
    }
}

pub struct SimpleSubscriber<const M: usize> {
    incoming: IncBuf<M>,
    _outgoing: OutBuf<M>,
}

impl<const M: usize> SimpleSubscriber<M> {
    #[must_use]
    pub const fn new(incoming: IncBuf<M>, outgoing: OutBuf<M>) -> Self {
        Self {
            incoming,
            _outgoing: outgoing,
        }
    }
}

impl<const M: usize> Subscriber for SimpleSubscriber<M> {
    fn notify(&mut self) {
        while let Some(msg) = self.incoming.pop() {
            let val =
                unsafe { std::ptr::read_unaligned(std::ptr::from_ref(&msg).cast::<[u64; 4]>()) };
            std::hint::black_box(val);
        }
    }
}

pub const MAX_MESSAGES: usize = 1024;
define_bus_config! {
    Legacy,
    max_subscribers: 32,
    max_messages: 1024,
    max_groups: 64
}

pub type BenchBus<C, F, const M: usize> = MessageBus<C, CreamyDriver, M, Subscribers<F, M>>;

pub fn init_bus_custom<C: BusConfig, F: SenderFn<M>, const M: usize>(
    indices: Vec<u8>,
    function: F,
    subs: usize,
) -> Result<BenchBus<C, F, M>, Box<dyn std::error::Error>> {
    let mut bus = BenchBus::new(CreamyDriver::new::<C>());
    let sender_id =
        bus.add_subscriber(move |inc, out| BenchmarkSender::new(out, inc, indices, function))?;

    let driver = bus.get_driver_mut();
    driver.declare_protocols("benchmark", sender_id);
    driver.sync_protocol_table(sender_id.as_u8(), vec!["benchmark".to_string()]);

    bus.update_lookup_table(sender_id);

    for _ in 0..subs {
        let listener_id = bus.add_subscriber(SimpleSubscriber::new)?;
        bus.get_driver_mut()
            .sync_protocol_table(listener_id.as_u8(), vec!["benchmark".to_string()]);
        bus.update_lookup_table(listener_id);
    }

    Ok(bus)
}

pub fn init_bus_legacy<F: SenderFn<MAX_MESSAGES>>(
    indices: Vec<u8>,
    function: F,
    subs: usize,
) -> Result<BenchBus<Legacy, F, MAX_MESSAGES>, Box<dyn std::error::Error>> {
    init_bus_custom::<Legacy, F, MAX_MESSAGES>(indices, function, subs)
}

const MAX_MESSAGES_LARGE: usize = 16000;
define_bus_config! {
    LegacyLargeBuffer,
    max_subscribers: 32,
    max_messages: 16000,
    max_groups: 128,
}

pub fn init_bus_legacy_large_buf<F: SenderFn<MAX_MESSAGES_LARGE>>(
    indices: Vec<u8>,
    function: F,
    subs: usize,
) -> Result<BenchBus<LegacyLargeBuffer, F, MAX_MESSAGES_LARGE>, Box<dyn std::error::Error>> {
    init_bus_custom::<LegacyLargeBuffer, F, MAX_MESSAGES_LARGE>(indices, function, subs)
}

const MAX_MESSAGES_ULTRA_LARGE: usize = 500_000;
define_bus_config! {
    LegacyUltraLargeBuffer,
    max_subscribers: 32,
    max_messages: 500_000,
    max_groups: 128,
}

pub fn init_bus_legacy_ularge_buf<F: SenderFn<MAX_MESSAGES_ULTRA_LARGE>>(
    indices: Vec<u8>,
    function: F,
    subs: usize,
) -> Result<BenchBus<LegacyUltraLargeBuffer, F, MAX_MESSAGES_ULTRA_LARGE>, Box<dyn std::error::Error>>
{
    init_bus_custom::<LegacyUltraLargeBuffer, F, MAX_MESSAGES_ULTRA_LARGE>(indices, function, subs)
}
