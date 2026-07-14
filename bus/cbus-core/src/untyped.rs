use core::fmt::Debug;

use static_assertions::assert_eq_size;

use crate::{defines::MESSAGE_SIZE, message::TypedMessage};

assert_eq_size!(UntypedMessage, [u8; MESSAGE_SIZE]);

#[repr(C, align(32))]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(unused)]
pub struct UntypedMessage {
    pub dst: u8,
    pub group: u8,
    pub src: u8,
    pub kind: u8,
    pub payload: [u8; 28],
}

impl TypedMessage for UntypedMessage {
    fn dst(&self) -> u8 {
        self.dst
    }

    fn with_dst(&mut self, dst: u8) -> &mut Self {
        self.dst = dst;
        self
    }

    fn src(&self) -> u8 {
        self.src
    }

    fn group(&self) -> u8 {
        self.group
    }

    fn with_group(&mut self, group: u8) -> &mut Self {
        self.group = group;
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

impl UntypedMessage {
    #[must_use]
    #[inline]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn new(dst: u8, group: u8, kind: u8, payload: [u8; 28]) -> Self {
        Self {
            dst,
            group,
            src: 0,
            kind,
            payload,
        }
    }

    #[must_use]
    #[inline]
    pub const fn cast_ref<M: TypedMessage>(&self) -> &M {
        const {
            assert!(
                size_of::<UntypedMessage>() == size_of::<M>(),
                "Invalid message size"
            );
            assert!(align_of::<UntypedMessage>() == align_of::<M>());
        }

        unsafe { &*core::ptr::from_ref::<UntypedMessage>(self).cast::<M>() }
    }

    #[must_use]
    #[inline]
    pub const fn cast<M: TypedMessage>(self) -> M {
        const {
            assert!(
                size_of::<UntypedMessage>() == size_of::<M>(),
                "Invalid message size"
            );
            assert!(
                align_of::<UntypedMessage>() == align_of::<M>(),
                "Invalid align"
            );
        }

        unsafe { *core::ptr::from_ref(&self).cast::<M>() }
    }
}
