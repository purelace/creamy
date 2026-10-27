use core::mem::MaybeUninit;

use cbus_core::{
    Subscriber,
    buffer::{IncBuf, OutBuf, reference::RefMutBuf},
};

#[derive(Debug)]
pub struct SubscriberData<S: Subscriber, const M: usize> {
    pub(super) incoming: IncBuf<M>,
    pub(super) outgoing: OutBuf<M>,
    pub(super) sub: MaybeUninit<S>,
}

impl<S: Subscriber, const M: usize> SubscriberData<S, M> {
    pub(super) fn null() -> Self {
        Self {
            incoming: IncBuf::null(),
            outgoing: OutBuf::null(),
            sub: MaybeUninit::zeroed(),
        }
    }

    /// Write
    pub fn incoming_mut(&mut self) -> RefMutBuf<'_, M> {
        unsafe { self.incoming.as_mut_buf() }
    }

    /// Read
    pub fn outgoing_mut(&mut self) -> RefMutBuf<'_, M> {
        unsafe { self.outgoing.as_mut_buf() }
    }

    pub const fn outgoing_count(&self) -> u32 {
        self.outgoing.count()
    }

    #[cfg(target_feature = "sse4.1")]
    pub const unsafe fn incoming_raw_ptr(&mut self) -> cbus_core::buffer::reference::RawBuf {
        unsafe { self.incoming.as_raw_buf() }
    }

    pub(super) unsafe fn drop(&mut self, is_initialized: bool) {
        unsafe {
            core::ptr::drop_in_place(core::ptr::from_mut(&mut self.incoming));
            core::ptr::drop_in_place(core::ptr::from_mut(&mut self.outgoing));
            if is_initialized {
                core::ptr::drop_in_place(self.sub.as_mut_ptr());
            }
        }
    }
}
