use std::ops::RangeInclusive;

use idmint::StackMint;

use crate::{
    BusDriver, BusError,
    config::{BusConfig, ValidConfig},
    core::{
        Subscriber, SubscriberId,
        buffer::{Incoming, Outgoing, Read, Write},
    },
    cpu::MemoryPools,
    defines::MESSAGE_SIZE,
    driver::Driver,
    sys::{BufferPool, MessagePool},
};

struct SubscriberData {
    id: u8,
    incoming: Incoming,
    outgoing: Outgoing,
}

pub struct MessageBus<D, S = Box<dyn Subscriber>>
where
    D: BusDriver,
    S: Subscriber,
{
    pool: MessagePool,

    write_pool: BufferPool<Read>,
    read_pool: BufferPool<Write>,

    subscribers: Vec<Option<S>>,
    uninit: Vec<(SubscriberId, S)>,

    remove_requests: Vec<SubscriberId>,
    removed: Vec<S>,

    mint: StackMint,

    driver: Driver<D>,
}

//impl<S: Subscriber> Default for MessageBus<(), S> {
//    /// Creates a new message bus with default `Advanced` configuration and no driver.
//    /// # Panics
//    /// Panics if the default configuration `Advanced` is invalid.
//    fn default() -> Self {
//        Self::Self::new(Advanced, |_, _| ()).expect("fbus: config must be valid")
//    }
//}

impl<D, S> MessageBus<D, S>
where
    D: BusDriver,
    S: Subscriber,
{
    /// Creates a new message bus with the specified configuration and driver.
    /// # Errors
    /// * [`BusError::ValueTooSmall`] — if a value is below the minimum required.
    /// * [`BusError::ValueOutOfRange`] — if a value exceeds the architectural limits.
    /// * [`BusError::PoolExhausted`] - if a pool is exhausted.
    pub fn new<C: BusConfig>(
        config: C,
        driver: impl Fn(&ValidConfig<C>, Outgoing) -> D,
    ) -> Result<Self, BusError> {
        let config = config.into_valid()?;

        let max_subscribers = config.max_subscribers() as usize;
        let max_messages = config.max_messages();

        let mut write_pool = BufferPool::new(max_subscribers, max_messages);
        let mut read_pool = BufferPool::new(max_subscribers, max_messages);

        let Some(outgoing) = read_pool.next_out() else {
            unreachable!();
        };

        // Этот буфер используется в качестве мусорки
        let Some(_) = write_pool.next_inc() else {
            // Буферы должны нарезаться одинакого. Если буферы не нарезаются одинаково, то это ошибка.
            unreachable!("Buffers must be sliced equally");
        };

        let driver = Driver::new(
            driver(&config, outgoing),
            config.max_groups(),
            config.max_subscribers(),
        );

        let subscribers = std::iter::repeat_with(|| None)
            .take(max_subscribers)
            .collect::<Vec<_>>();

        Ok(Self {
            pool: MessagePool::new(max_subscribers * max_messages * MESSAGE_SIZE),
            write_pool,
            read_pool,
            subscribers,
            uninit: Vec::with_capacity(32),
            remove_requests: vec![],
            removed: vec![],
            mint: StackMint::new(1), // Reserve for the zero subscriber
            driver,
        })
    }

    /// Adds a new subscriber
    ///
    /// # Errors
    /// Returns an error if the pool is exhausted.
    pub fn add_subscriber<R: Into<S>>(
        &mut self,
        f: impl FnOnce(Incoming, Outgoing) -> R,
    ) -> Result<SubscriberId, BusError> {
        let data = self.get_subscriber_data()?;
        let id = SubscriberId::new(data.id);
        self.uninit
            .push((id, f(data.incoming, data.outgoing).into()));

        Ok(id)
    }

    fn get_subscriber_data(&mut self) -> Result<SubscriberData, BusError> {
        let id = self.mint.issue().unwrap();
        let Some(outgoing) = self.read_pool.next_out() else {
            let max = self.read_pool.capacity();
            return Err(BusError::PoolExhausted { max });
        };

        let Some(incoming) = self.write_pool.next_inc() else {
            // Буферы должны нарезаться одинакого. Если буферы не нарезаются одинаково, то это ошибка.
            unreachable!("Buffers must be sliced equally");
        };

        Ok(SubscriberData {
            id,
            incoming,
            outgoing,
        })
    }

    fn init_subscribers(&mut self) {
        let uninit = std::mem::take(&mut self.uninit);
        for (id, sub) in uninit {
            assert!(
                self.subscribers[id.cast_usize()].is_none(),
                "Invalid operation: cannot replace subscriber"
            );
            self.subscribers[id.cast_usize()] = Some(sub);

            self.driver.on_subscribe(id.u8());
        }
    }

    /// # Errors
    ///
    /// This function will return an error if the subscriber ID is zero or if a remove request has already been sent.
    pub fn send_remove_request(&mut self, id: SubscriberId) -> Result<(), BusError> {
        if id == SubscriberId::new(0) {
            return Err(BusError::InvalidSubscriberId);
        }

        if self.remove_requests.contains(&id) {
            return Err(BusError::RequestAlreadySent);
        }

        if !self.mint.is_value_in_use(id.u8()) {
            return Err(BusError::SubscriberNotRegistered);
        }

        self.remove_requests.push(id);
        Ok(())
    }

    pub fn removed(&mut self) -> Vec<S> {
        std::mem::take(&mut self.removed)
    }

    fn handle_remove_requests(&mut self) {
        for id in self.remove_requests.drain(..) {
            self.driver.on_unsubscribe(id.u8());

            self.mint.recycle(*id);

            let id = id.cast_usize();
            self.read_pool.return_buffer(id);
            self.write_pool.return_buffer(id);
            let subscriber = self.subscribers.get_mut(id).unwrap().take().unwrap();
            self.removed.push(subscriber);
        }

        //TODO last_id
        //TODO buffer recycle
    }

    // Попробовать подготовить данные так, чтобы можно было одной операцией их отправить
    // Обрабатывать возможное переполнение

    const fn subscriber_range(&self) -> RangeInclusive<usize> {
        //TODO fix
        0..=self.mint.last() as usize
    }

    pub fn tick(&mut self) {
        self.handle_remove_requests();
        self.init_subscribers();

        let range = self.subscriber_range();
        let memory = MemoryPools {
            write: &mut self.write_pool,
            read: &mut self.read_pool,
            message: &mut self.pool,
        };

        self.driver.process_messages(memory, range);
        self.subscribers[1..=self.mint.last() as usize]
            .iter_mut()
            .flatten()
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
impl<D, S> MessageBus<D, S>
where
    D: BusDriver,
    S: Subscriber,
{
    pub const fn subscribers(&self) -> usize {
        self.mint.used() as usize
    }

    pub const fn get_driver_mut(&mut self) -> &mut D {
        self.driver.get_inner_mut()
    }

    pub const fn get_driver(&self) -> &D {
        self.driver.get_inner()
    }
}
