use core::fmt::Debug;

pub trait TypedMessage: Debug + Copy + Clone {
    fn dst(&self) -> u8;
    fn with_dst(&mut self, dst: u8) -> &mut Self;
    fn src(&self) -> u8;
    fn group(&self) -> u8;
    fn with_group(&mut self, group: u8) -> &mut Self;
    fn kind(&self) -> u8;
    fn with_kind(&mut self, kind: u8) -> &mut Self;

    #[must_use]
    #[inline]
    fn cast<M: TypedMessage>(self) -> M {
        const {
            assert!(size_of::<Self>() == size_of::<M>());
            assert!(align_of::<Self>() == align_of::<M>());
        }
        unsafe { *core::ptr::from_ref(&self).cast::<M>() }
    }
}
