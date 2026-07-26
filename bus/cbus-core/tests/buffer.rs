use core::ptr::NonNull;

use cbus_core::{
    UntypedMessage,
    buffer::SharedBuf,
    defines::{MESSAGE_SIZE, METADATA},
};

#[test]
fn shared_ref() {
    let buf = SharedBuf::<1024>::new();
    assert_eq!(buf.references(), 1);
    let buf0 = buf.clone();
    assert_eq!(buf.references(), 2);
    assert_eq!(buf0.references(), 2);

    drop(buf);

    assert_eq!(buf0.references(), 1);
}

#[test]
#[should_panic = "zaebal"]
fn shared_ref_in_vec() {
    let mut i = 0;
    let vec = core::iter::repeat_with(|| {
        assert!(i != 12, "zaebal");
        i += 1;
        SharedBuf::<1024>::new()
    })
    .take(24)
    .collect::<Vec<_>>();

    core::hint::black_box(&vec);

    let vec0 = vec.clone();
    drop(vec);
    core::hint::black_box(&vec0);
}

const fn cast(array: [u8; MESSAGE_SIZE]) -> UntypedMessage {
    unsafe { core::mem::transmute(array) }
}

#[test]
fn shared_ref_from_slice() {
    const A: UntypedMessage = cast([0; MESSAGE_SIZE]);
    const B: UntypedMessage = cast([1; MESSAGE_SIZE]);
    const C: UntypedMessage = cast([2; MESSAGE_SIZE]);
    const D: UntypedMessage = cast([3; MESSAGE_SIZE]);

    #[repr(align(64))]
    struct FixedBuffer([u8; SIZE]);

    const M: usize = 4;
    const SIZE: usize = MESSAGE_SIZE * M + METADATA;
    let mut fixed_buf = FixedBuffer([0; SIZE]);
    let mut buf = unsafe {
        let ptr = NonNull::new_unchecked(fixed_buf.0.as_mut_ptr());
        SharedBuf::<M>::from_ptr(ptr, false)
    };

    let mut buf0 = buf.clone();

    let mut mut_buf = buf.as_mut_buf();
    let data = mut_buf.slice_mut();
    data[0] = A;
    data[1] = B;

    let mut mut_buf = buf0.as_mut_buf();
    let data = mut_buf.slice_mut();
    data[2] = C;
    data[3] = D;

    let mut buf = buf0.clone();

    assert_eq!(buf.as_mut_buf().slice_mut(), &mut [A, B, C, D]);
}
