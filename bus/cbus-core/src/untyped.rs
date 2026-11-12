use core::fmt::Debug;

use static_assertions::assert_eq_size;

use crate::{ProtocolId, SubscriberId, defines::MESSAGE_SIZE, message::TypedMessage};

assert_eq_size!(UntypedMessage, [u8; MESSAGE_SIZE]);

#[repr(C, align(32))]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct UntypedMessage {
    pub dst: u8,
    pub protocol: u8,
    pub src: u8,
    pub kind: u8,
    pub payload: [u8; 28],
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl TypedMessage for UntypedMessage {
    fn dst(&self) -> u8 {
        self.dst
    }

    fn with_dst(&mut self, dst: SubscriberId) -> &mut Self {
        self.dst = dst.as_u8();
        self
    }

    fn src(&self) -> u8 {
        self.src
    }

    fn protocol(&self) -> u8 {
        self.protocol
    }

    fn with_protocol(&mut self, protocol: ProtocolId) -> &mut Self {
        self.protocol = protocol.as_u8();
        self
    }

    fn kind(&self) -> u8 {
        self.kind
    }

    fn with_kind(&mut self, kind: u8) -> &mut Self {
        self.kind = kind;
        self
    }
}
