use alloc::boxed::Box;

use cbus_core::{
    SubscriberId,
    buffer::{IncBuf, OutBuf},
};

use crate::{
    BusDriver, BusError,
    config::BusConfig,
    core::Subscriber,
    cpu::MemoryPools,
    driver::Driver,
    sys::{MessagePool, subscriber::SubscriberList},
};

#[derive(Debug)]
pub struct MessageBus<C, D, const M: usize, S = Box<dyn Subscriber>>
where
    C: BusConfig,
    D: BusDriver,
    S: Subscriber,
{
    pool: MessagePool<C>,
    subscribers: SubscriberList<C, S, M>,
    driver: Driver<C, D, S, M>,
}

impl<C, D, const M: usize, S> MessageBus<C, D, M, S>
where
    C: BusConfig,
    D: BusDriver,
    S: Subscriber,
{
    /// Creates a new message bus with the specified configuration and driver.
    pub fn new(driver: D) -> Self {
        const {
            assert!(C::MAX_MESSAGES.get() as usize == M);
        };

        let driver = Driver::new(driver);

        Self {
            pool: MessagePool::new(),
            subscribers: SubscriberList::default(),
            driver,
        }
    }

    pub fn update_lookup_table(&mut self, id: SubscriberId) {
        self.driver.on_subscribe(id);
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, name = "bus::tick", level = "trace")
    )]
    pub fn tick(&mut self) {
        let range = self.subscribers.subscriber_range();
        let memory = MemoryPools {
            subscribers: &mut self.subscribers,
            message: &mut self.pool,
        };

        self.driver.process_messages(memory, range);

        core::hint::black_box(());

        //self.subscribers[1..=self.mint.last() as usize]
        self.subscribers.iter_mut().for_each(|sub| {
            cfg_select! {
                feature = "tracing" => {
                    let span = tracing::span!(tracing::Level::INFO, "subscriber::notify()");
                    let _enter = span.enter();
                }
                _ => {}
            }
            sub.notify();
        });
    }

    pub fn full_tick(&mut self) {
        self.tick();
        self.tick();
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<C, D, const M: usize, S> MessageBus<C, D, M, S>
where
    C: BusConfig,
    D: BusDriver,
    S: Subscriber,
{
    /// Adds a new subscriber with specified buffers
    ///
    /// # Errors
    /// Returns an error if the pool is exhausted.
    pub fn add_subscriber_with<R: Into<S>>(
        &mut self,
        inc: IncBuf<M>,
        out: OutBuf<M>,
        sub: R,
    ) -> Result<SubscriberId, BusError> {
        self.subscribers.add_with(inc, out, sub).inspect(|&id| {
            self.driver.on_subscribe(id);
        })
    }

    /// Adds a new subscriber
    ///
    /// # Errors
    /// Returns an error if the pool is exhausted.
    pub fn add_subscriber<R: Into<S>>(
        &mut self,
        sub: impl FnOnce(IncBuf<M>, OutBuf<M>) -> R,
    ) -> Result<SubscriberId, BusError> {
        self.subscribers.add(sub).inspect(|&id| {
            self.driver.on_subscribe(id);
        })
    }

    /// # Errors
    ///
    /// This function will return an error if the subscriber is not registered
    pub fn remove_subscriber(&mut self, id: SubscriberId) -> Result<S, BusError> {
        self.subscribers.remove(id).inspect(|_| {
            self.driver.on_unsubscribe(id);
        })
    }

    pub fn get_subscriber(&self, id: SubscriberId) -> Option<&S> {
        self.subscribers.get(id.as_u8())
    }

    pub fn get_subscriber_mut(&mut self, id: SubscriberId) -> Option<&mut S> {
        self.subscribers.get_mut(id)
    }

    pub const fn subscribers(&self) -> u8 {
        self.subscribers.count()
    }

    pub const fn get_driver_mut(&mut self) -> &mut D {
        self.driver.get_inner_mut()
    }

    pub const fn get_driver(&self) -> &D {
        self.driver.get_inner()
    }
}
