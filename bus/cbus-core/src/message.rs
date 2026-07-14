use core::fmt::Debug;

//pub trait Payload: MessageSuper {
//    fn as_bytes(&self) -> &[u8; 28] {
//        bytemuck::cast_ref(self)
//    }
//}

//pub trait MessageSuper: Debug + Copy + Clone {}

pub trait TypedMessage: Debug + Copy + Clone {
    fn dst(&self) -> u8;
    fn with_dst(&mut self, dst: u8) -> &mut Self;
    fn src(&self) -> u8;
    fn group(&self) -> u8;
    fn with_group(&mut self, group: u8) -> &mut Self;
    fn kind(&self) -> u8;
    fn with_kind(&mut self, kind: u8) -> &mut Self;

    //fn as_raw_bytes(&self) -> &[u8; MESSAGE_SIZE];
    //fn payload_as_raw_bytes(&self) -> &[u8; PAYLOAD_SIZE];

    #[must_use]
    #[inline(always)]
    fn cast<M: TypedMessage>(self) -> M {
        const {
            assert!(size_of::<Self>() == size_of::<M>());
            assert!(align_of::<Self>() == align_of::<M>());
        }
        unsafe { *core::ptr::from_ref(&self).cast::<M>() }
    }
}
