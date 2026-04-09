use core::fmt::Debug;

use bytemuck::{Pod, Zeroable};
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
    pub version: u8,
    pub payload: [u8; 27],
}

unsafe impl Zeroable for UntypedMessage {}
unsafe impl Pod for UntypedMessage {}

impl UntypedMessage {
    #[must_use]
    #[inline]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn new(dst: u8, group: u8, kind: u8, version: u8, payload: [u8; 27]) -> Self {
        Self {
            dst,
            src: 0,
            version,
            kind,
            payload,
            group,
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
