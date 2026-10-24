use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BusError {
    #[error("Cannot remove subscriber with id == 0")]
    InvalidSubscriberId,

    #[error("Subscriber not registered")]
    SubscriberNotRegistered,

    #[error("Cannot allocate buffer. Pool is full. Possible count of subscribers: {max}")]
    PoolExhausted { max: u8 },
}
