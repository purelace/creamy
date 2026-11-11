use cbus_core::GroupId;
pub use cbus_core::{
    UntypedMessage,
    message::{TypedMessage, cast_message},
};

pub trait Message: TypedMessage {
    const GROUP: GroupId;
    const KIND: u8;
}

pub trait MessageContent {
    type Message: Message;
}
