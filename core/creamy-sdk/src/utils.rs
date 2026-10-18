use cbus_core::{
    defines::{MESSAGE_SIZE, PAYLOAD_SIZE},
    message::TypedMessage,
};

pub const fn extract_payload<M: TypedMessage, P>(src: &M) -> P {
    const {
        assert!(
            size_of::<M>() == MESSAGE_SIZE,
            "Source structure must be exactly 32 bytes"
        );
        assert!(
            size_of::<P>() == PAYLOAD_SIZE,
            "Target structure must be exactly 28 bytes"
        );
    }

    unsafe {
        let src_ptr = core::ptr::from_ref::<M>(src).cast::<u8>();
        let payload_ptr = src_ptr.add(4).cast::<P>();
        payload_ptr.read_unaligned()
    }
}
