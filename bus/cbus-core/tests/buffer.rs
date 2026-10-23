use cbus_core::{UntypedMessage, buffer::IncBuf};

const SAMPLE: UntypedMessage = UntypedMessage {
    dst: 123,
    group: 3,
    src: 107,
    kind: 9,
    payload: [1; 28],
};

fn write_messages<const M: usize>(buf: &mut IncBuf<M>, count: u32) {
    let mut buf = unsafe { buf.as_mut_buf() };
    let slice = buf.write_slice_mut(count);
    slice.fill(SAMPLE);

    // SAFETY: not safety
    unsafe {
        buf.add_count(count);
    }
}

#[test]
fn safe_read_write() {
    let mut inc = IncBuf::<1024>::new();
    write_messages(&mut inc, 128);
    assert_eq!(inc.count(), 128);

    let mut index = 0;
    for message in inc.pop_all() {
        index += 1;
        assert_eq!(*message, SAMPLE);
    }
    assert_eq!(index, 128);
    assert_eq!(inc.count(), 0);

    write_messages(&mut inc, 128);
    assert_eq!(inc.count(), 128);

    index = 0;
    while let Some(message) = inc.pop() {
        index += 1;
        assert_eq!(message, SAMPLE);
    }
    assert_eq!(index, 128);
    assert_eq!(inc.count(), 0);
}

#[test]
fn safe_clear_and_try_get_message() {
    let mut inc = IncBuf::<1024>::default();
    write_messages(&mut inc, 1024);
    assert_eq!(inc.count(), 1024);

    inc.clear();
    assert_eq!(inc.count(), 0);

    assert!(inc.pop().is_none());
    assert_eq!(inc.pop_all().count(), 0);
}

#[test]
#[should_panic(expected = "range end index 1152 out of range for slice of length 1024")]
fn out_of_bounds() {
    let mut inc = IncBuf::<1024>::default();
    write_messages(&mut inc, 1024);
    assert_eq!(inc.count(), 1024);

    write_messages(&mut inc, 128);
    assert_eq!(inc.count(), 128);
}

#[test]
#[should_panic(expected = "new count exceeds buffer capacity")]
fn unsafe_add_operation() {
    let mut inc = IncBuf::<1024>::new();
    let mut buf = unsafe { inc.as_mut_buf() };

    // SAFETY: not safety
    unsafe {
        buf.add_count(2048);
    }
}

#[test]
fn unsafe_() {}
