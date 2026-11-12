use cbus_core::ProtocolId;
pub use cbus_core::{
    UntypedMessage,
    message::{TypedMessage, cast_message},
};

pub trait Message: TypedMessage {
    const PROTOCOL: ProtocolId;
    const KIND: u8;
}

pub trait MessageContent {
    type Message: Message;
}
