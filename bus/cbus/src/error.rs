use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BusError {
    #[error("Cannot remove subscriber with id == 0")]
    InvalidSubscriberId,

    #[error("Remove request already sent")]
    RequestAlreadySent,

    #[error("Subscriber not registered")]
    SubscriberNotRegistered,

    #[error("Cannot allocate buffer. Pool is full. Possible count of subscribers: {max}")]
    PoolExhausted { max: usize },

    #[error("Size of pool is too big ({current}). Size should be less than {max} bytes")]
    TooBigPoolSize { current: usize, max: usize },

    #[error("Config '{name}' is too small: {current}. Minimum required: {min}")]
    ValueTooSmall {
        name: String,
        current: usize,
        min: usize,
    },

    #[error("Config '{name}' is too big.")]
    ValueTooBig { name: String },

    #[error("Config '{name}' is out of range: {current}. Valid range: [{min}..{max}]")]
    ValueOutOfRange {
        name: String,
        current: usize,
        min: usize,
        max: usize,
    },

    #[error("Config '{name}' is not multiple of 2")]
    ValueIsNotMultipleOf2 { name: String },

    #[error("'max_groups' must be a power of 2")]
    GroupsIsNotPowerOf2,

    #[error("{0}")]
    DriverError(String),
}
