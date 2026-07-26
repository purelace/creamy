use alloc::boxed::Box;
use core::ops::RangeInclusive;

use cbus_core::{
    SubscriberId,
    buffer::{IncBuf, OutBuf, RawBuf, RefBuf, RefMutBuf},
};
use idmint::StackMint;

use crate::{
    BusDriver, BusError, config::BusConfig, core::Subscriber, cpu::MemoryPools, driver::Driver,
    sys::MessagePool,
};

#[derive(Debug)]
pub(crate) struct SubscriberData<S: Subscriber, const M: usize> {
    incoming: IncBuf<M>,
    outgoing: OutBuf<M>,
    sub: Option<S>,
}

impl<S: Subscriber, const M: usize> SubscriberData<S, M> {
    pub unsafe fn null() -> Self {
        //TODO: fix
        Self {
            incoming: IncBuf::default(),
            outgoing: OutBuf::default(),
            sub: None,
        }
    }

    /// Write
    pub const fn incoming_mut(&mut self) -> RefMutBuf<'_, M> {
        self.incoming.as_inner_mut().as_mut_buf()
    }

    /// Read
    pub const fn outgoing_mut(&mut self) -> RefMutBuf<'_, M> {
        self.outgoing.as_inner_mut().as_mut_buf()
    }

    pub const fn outgoing_ref(&self) -> RefBuf<'_, M> {
        self.outgoing.as_inner_ref().as_ref_buf()
    }

    pub const unsafe fn incoming_raw_ptr(&mut self) -> RawBuf {
        unsafe { self.incoming.as_inner_mut().as_raw_buf() }
    }
}

#[derive(Debug)]
pub struct MessageBus<C, D, const M: usize, S = Box<dyn Subscriber>>
where
    C: BusConfig,
    D: BusDriver,
    S: Subscriber,
{
    pool: MessagePool<C>,
    subscribers: Box<[SubscriberData<S, M>]>,
    mint: StackMint,
    driver: Driver<C, D, S, M>,
}

impl<C, D, const M: usize, S> MessageBus<C, D, M, S>
where
    C: BusConfig,
    D: BusDriver,
    S: Subscriber,
{
    /// Creates a new message bus with the specified configuration and driver.
    /// # Errors
    /// * [`BusError::ValueTooSmall`] — if a value is below the minimum required.
    /// * [`BusError::ValueOutOfRange`] — if a value exceeds the architectural limits.
    /// * [`BusError::PoolExhausted`] - if a pool is exhausted.
    pub fn new(driver: D) -> Self {
        let driver = Driver::new(driver);

        let subscribers = core::iter::repeat_with(|| unsafe { SubscriberData::null() })
            .take(C::MAX_SUBSCRIBERS.get() as usize)
            .collect::<Box<[_]>>();

        Self {
            pool: MessagePool::new(),
            subscribers,
            mint: StackMint::new(1), // SubscriberId cannot be zero
            driver,
        }
    }

    fn new_id(&mut self) -> Result<SubscriberId, BusError> {
        let Some(id) = self.mint.issue() else {
            return Err(BusError::PoolExhausted {
                max: C::MAX_SUBSCRIBERS.get(),
            });
        };

        if id >= C::MAX_SUBSCRIBERS.get() {
            self.mint.recycle(id);
            return Err(BusError::PoolExhausted {
                max: C::MAX_SUBSCRIBERS.get(),
            });
        }

        Ok(unsafe { SubscriberId::new_unchecked(id) })
    }

    fn add_subscriber_unchecked(
        &mut self,
        id: SubscriberId,
        inc: IncBuf<M>,
        out: OutBuf<M>,
        sub: S,
    ) {
        //let len = C::MAX_MESSAGES.get() as usize * MESSAGE_SIZE + METADATA;
        //assert!(
        //    data.sub.is_none(),
        //    "Invalid operation: cannot replace subscriber"
        //);

        let data = &mut self.subscribers[id.as_usize()];
        data.incoming = inc;
        data.outgoing = out;
        data.sub = Some(sub);

        self.driver.on_subscribe(id);
    }

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
        let id = self.new_id()?;
        self.add_subscriber_unchecked(id, inc, out, sub.into());
        Ok(id)
    }

    /// Adds a new subscriber
    ///
    /// # Errors
    /// Returns an error if the pool is exhausted.
    pub fn add_subscriber<R: Into<S>>(
        &mut self,
        sub: impl FnOnce(IncBuf<M>, OutBuf<M>) -> R,
    ) -> Result<SubscriberId, BusError> {
        let id = self.new_id()?;

        let inc = IncBuf::default();
        let out = OutBuf::default();

        let sub = sub(inc.clone(), out.clone()).into();

        self.add_subscriber_unchecked(id, inc, out, sub);

        Ok(id)
    }

    /// # Errors
    ///
    /// This function will return an error if the subscriber ID is zero or a subscriber is not registered
    pub fn remove_subscriber(&mut self, id: SubscriberId) -> Result<S, BusError> {
        let Some(data) = self.subscribers.get_mut(id.as_usize()) else {
            return Err(BusError::InvalidSubscriberId);
        };

        let Some(sub) = data.sub.take() else {
            return Err(BusError::SubscriberNotRegistered);
        };

        self.driver.on_unsubscribe(id);
        self.mint.recycle(id.as_u8());

        Ok(sub)
    }

    pub fn update_lookup_table(&mut self, id: SubscriberId) {
        self.driver.on_subscribe(id);
    }

    // Попробовать подготовить данные так, чтобы можно было одной операцией их отправить
    // Обрабатывать возможное переполнение

    const fn subscriber_range(&self) -> RangeInclusive<u8> {
        0..=self.mint.last()
    }

    #[tracing::instrument(skip_all)]
    pub fn tick(&mut self) {
        let range = self.subscriber_range();
        let memory = MemoryPools {
            subscribers: &mut self.subscribers,
            message: &mut self.pool,
        };

        self.driver.process_messages(memory, range);
        self.subscribers[1..=self.mint.last() as usize]
            .iter_mut()
            .flat_map(|d| &mut d.sub)
            .for_each(|sub| {
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
    pub const fn subscribers(&self) -> u8 {
        self.mint.used() - 1
    }

    pub const fn get_driver_mut(&mut self) -> &mut D {
        self.driver.get_inner_mut()
    }

    pub const fn get_driver(&self) -> &D {
        self.driver.get_inner()
    }
}
