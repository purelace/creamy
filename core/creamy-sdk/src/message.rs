use core::num::NonZeroU8;

pub use cbus_core::{
    UntypedMessage,
    message::{TypedMessage, cast_message},
};

pub trait Message: TypedMessage {
    const GROUP: NonZeroU8;
    const KIND: u8;
}

pub trait MessageContent {
    type Message: Message;
}
