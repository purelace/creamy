use alloc::boxed::Box;
use core::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

use as_guard::AsGuard;
use cbus_core::{
    Subscriber, SubscriberId,
    buffer::{IncBuf, OutBuf, reference::RefMutBuf},
};
use idmint::StackMint;

use super::data::SubscriberData;
use crate::{BusError, config::BusConfig, sys::SubscriberRange};

#[derive(Default, Clone, Copy)]
pub struct Mask([u8; 32]);

impl core::fmt::Debug for Mask {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let iter = (0u8..=255u8).filter(|position| {
            let bit = self.get_bit(*position);
            bit != 0
        });
        f.debug_list().entries(iter).finish()
    }
}

impl Mask {
    const fn get_byte_with_mask(position: u8) -> (u8, u8) {
        let byte = position / 8;
        let offset = byte * 8;
        let bit = position - offset;
        (byte, 0b_0000_0001u8 << bit)
    }

    const fn unset_bit(&mut self, position: u8) {
        let (byte, mask) = Self::get_byte_with_mask(position);
        let byte = &mut self.0[byte as usize];
        *byte &= !mask;
    }

    const fn set_bit(&mut self, position: u8) {
        let (byte, mask) = Self::get_byte_with_mask(position);
        let byte = &mut self.0[byte as usize];
        *byte |= mask;
    }

    pub const fn get_bit(&self, position: u8) -> u8 {
        let (byte, mask) = Self::get_byte_with_mask(position);
        let byte = &self.0[byte as usize];
        *byte & mask
    }
}

#[derive(Debug)]
pub struct SubscriberList<C: BusConfig, S: Subscriber, const M: usize> {
    subscribers: Box<[ManuallyDrop<SubscriberData<S, M>>]>,
    mint: StackMint,
    mask: Mask,
    _phantom: PhantomData<C>,
}

impl<C: BusConfig, S: Subscriber, const M: usize> Default for SubscriberList<C, S, M> {
    fn default() -> Self {
        let subscribers = core::iter::repeat_with(|| ManuallyDrop::new(SubscriberData::null()))
            .take(C::MAX_SUBSCRIBERS.get() as usize)
            .collect::<Box<[_]>>();

        Self {
            subscribers,
            mint: StackMint::new(1), // SubscriberId cannot be zero
            mask: Mask::default(),
            _phantom: PhantomData,
        }
    }
}

impl<C: BusConfig, S: Subscriber, const M: usize> SubscriberList<C, S, M> {
    pub const fn get_mask(&self) -> Mask {
        self.mask
    }

    pub const fn subscriber_range(&self) -> SubscriberRange {
        const ONE: SubscriberId = SubscriberId::new_u8(1).unwrap();
        //TODO: get first
        SubscriberRange::new(ONE, self.mint.last())
    }

    fn new_id(&mut self) -> Result<SubscriberId, BusError> {
        let Some(id) = self.mint.issue() else {
            return Err(BusError::PoolExhausted {
                max: C::MAX_SUBSCRIBERS.get(),
            });
        };

        if id >= C::MAX_SUBSCRIBERS.get() {
            self.mint.recycle(id);
            return Err(BusError::PoolExhausted {
                max: C::MAX_SUBSCRIBERS.get(),
            });
        }

        Ok(unsafe { SubscriberId::new_unchecked(id) })
    }

    fn add_subscriber_unchecked(
        &mut self,
        id: SubscriberId,
        inc: IncBuf<M>,
        out: OutBuf<M>,
        sub: S,
    ) {
        self.mask.set_bit(id.as_u8());
        self.subscribers[id.as_usize()] = ManuallyDrop::new(SubscriberData {
            incoming: inc,
            outgoing: out,
            sub: MaybeUninit::new(sub),
        });
    }

    pub fn add_with<R: Into<S>>(
        &mut self,
        inc: IncBuf<M>,
        out: OutBuf<M>,
        sub: R,
    ) -> Result<SubscriberId, BusError> {
        let id = self.new_id()?;
        self.add_subscriber_unchecked(id, inc, out, sub.into());
        Ok(id)
    }

    /// Adds a new subscriber
    ///
    /// # Errors
    /// Returns an error if the pool is exhausted.
    pub fn add<R: Into<S>>(
        &mut self,
        sub: impl FnOnce(IncBuf<M>, OutBuf<M>) -> R,
    ) -> Result<SubscriberId, BusError> {
        let id = self.new_id()?;

        let inc = IncBuf::default();
        let out = OutBuf::default();

        let sub = sub(inc.clone(), out.clone()).into();

        self.add_subscriber_unchecked(id, inc, out, sub);

        Ok(id)
    }

    /// # Errors
    ///
    /// This function will return an error if the subscriber is not registered
    pub fn remove(&mut self, id: SubscriberId) -> Result<S, BusError> {
        if id.as_usize() > self.subscribers.len() {
            return Err(BusError::InvalidSubscriberId);
        }

        let position = id.as_u8();
        if self.mask.get_bit(position) == 0 {
            return Err(BusError::SubscriberNotRegistered);
        }

        self.mask.unset_bit(position);
        self.mint.recycle(position);

        let value = &mut self.subscribers[position as usize];
        let mut data = core::mem::replace(value, ManuallyDrop::new(SubscriberData::null()));
        let sub = core::mem::replace(&mut data.sub, MaybeUninit::zeroed());
        unsafe { data.drop(false) };

        Ok(unsafe { sub.assume_init() })
    }

    pub fn get_data(&self, id: u8) -> Option<&SubscriberData<S, M>> {
        if self.mask.get_bit(id) == 0 {
            return None;
        }

        self.subscribers.get(id as usize).map(|d| &**d)
    }

    pub fn get(&self, id: u8) -> Option<&S> {
        self.get_data(id)
            .map(|data| unsafe { data.sub.assume_init_ref() })
    }

    pub fn get_data_mut(&mut self, id: u8) -> Option<&mut SubscriberData<S, M>> {
        if self.mask.get_bit(id) == 0 {
            return None;
        }

        self.subscribers.get_mut(id as usize).map(|d| &mut **d)
    }

    pub fn get_mut(&mut self, id: SubscriberId) -> Option<&mut S> {
        self.get_data_mut(id.as_u8())
            .map(|data| unsafe { data.sub.assume_init_mut() })
    }

    pub unsafe fn get_data_unchecked(&self, id: u8) -> &SubscriberData<S, M> {
        debug_assert!((id as usize) < self.subscribers.len());
        unsafe { self.subscribers.get_unchecked(id as usize) }
    }

    pub unsafe fn get_data_mut_unchecked(&mut self, id: u8) -> &mut SubscriberData<S, M> {
        debug_assert!((id as usize) < self.subscribers.len());
        unsafe { self.subscribers.get_unchecked_mut(id as usize) }
    }

    pub const fn count(&self) -> u8 {
        self.mint.used() - 1
    }

    pub fn iter(&self) -> ListIter<'_, S, M> {
        ListIter {
            subscribers: self.subscribers[1..].iter(),
            mask: self.mask,
            position: Some(1),
        }
    }

    pub fn iter_mut(&mut self) -> ListIterMut<'_, S, M> {
        ListIterMut {
            subscribers: self.subscribers[1..].iter_mut(),
            mask: self.mask,
            position: Some(1),
        }
    }

    pub unsafe fn get_mut_inc_buf(&mut self, index: u8) -> RefMutBuf<'_, M> {
        unsafe {
            let ptr = self.subscribers.as_mut_ptr().add(index as usize);
            (*ptr).incoming_mut()
        }
    }

    pub unsafe fn get_mut_out_buf<'a>(&mut self, index: u8) -> RefMutBuf<'a, M> {
        unsafe {
            let ptr = self.subscribers.as_mut_ptr().add(index as usize);
            (*ptr).outgoing_mut()
        }
    }
}

impl<C: BusConfig, S: Subscriber, const M: usize> Drop for SubscriberList<C, S, M> {
    fn drop(&mut self) {
        for position in 1..self.subscribers.len().safe_as::<u8>() {
            let is_initialized = self.mask.get_bit(position) != 0;

            unsafe {
                self.subscribers[position as usize].drop(is_initialized);
            }
        }
        //unsafe {
        //    let ptr = core::ptr::from_mut(&mut self.subscribers);
        //    core::ptr::drop_in_place(ptr);
        //}
    }
}

//TODO: use self.mint.last()
macro_rules! iterator {
    (
        $name: ident,
        {$( $mut_:tt )?},
        $init: ident
    ) => {
        impl<'a, S: Subscriber, const M: usize> Iterator for $name<'a, S, M> {
            type Item = &'a $( $mut_ )? S;

            fn next(&mut self) -> Option<Self::Item> {
                while let Some(maybe_uninit_data) = self.subscribers.next() {
                    let position = self.position?;
                    self.position = position.checked_add(1);

                    if self.mask.get_bit(position) == 0 {
                        continue;
                    }

                    let sub = unsafe { maybe_uninit_data.sub.$init() };

                    return Some(sub);
                }

                None
            }
        }
    };
}

pub struct ListIter<'a, S: Subscriber, const M: usize> {
    subscribers: core::slice::Iter<'a, ManuallyDrop<SubscriberData<S, M>>>,
    mask: Mask,
    position: Option<u8>,
}
iterator!(ListIter, {}, assume_init_ref);

pub struct ListIterMut<'a, S: Subscriber, const M: usize> {
    subscribers: core::slice::IterMut<'a, ManuallyDrop<SubscriberData<S, M>>>,
    mask: Mask,
    position: Option<u8>,
}
iterator!(ListIterMut, { mut }, assume_init_mut);

impl<'a, C: BusConfig, S: Subscriber, const M: usize> IntoIterator for &'a SubscriberList<C, S, M> {
    type Item = &'a S;

    type IntoIter = ListIter<'a, S, M>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, C: BusConfig, S: Subscriber, const M: usize> IntoIterator
    for &'a mut SubscriberList<C, S, M>
{
    type Item = &'a mut S;

    type IntoIter = ListIterMut<'a, S, M>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
