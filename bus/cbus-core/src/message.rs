use core::fmt::Debug;

pub trait TypedMessage: Debug + Copy + Clone {
    fn dst(&self) -> u8;
    fn with_dst(&mut self, dst: u8) -> &mut Self;
    fn src(&self) -> u8;
    fn group(&self) -> u8;
    fn with_group(&mut self, group: u8) -> &mut Self;
    fn kind(&self) -> u8;
    fn with_kind(&mut self, kind: u8) -> &mut Self;
}

#[inline]
pub const fn cast_message<T: TypedMessage, F: TypedMessage>(message: F) -> T {
    const {
        assert!(size_of::<F>() == size_of::<T>());
        assert!(align_of::<F>() == align_of::<T>());
    }
    unsafe { *core::ptr::from_ref(&message).cast::<T>() }
}
