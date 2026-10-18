use core::assert_matches;
use std::num::{NonZeroI16, NonZeroU8};

use creamy_binary::{BinaryReader, BinaryWriter, LittleEndian, ReadError};
use creamy_binary_derive::{BinRead, BinWrite};

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
struct Cache(Option<i128>);

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
pub struct Data {
    year: u16,
    month: NonZeroU8,
    day: u16,
    is_special: bool,
    cache: Cache,
    option_none: Option<()>,
    option_some: Option<()>,
    array: [i64; 16],
    char: char,
}

#[test]
fn serialization() -> Result<(), Box<dyn core::error::Error>> {
    let mut buffer = Vec::with_capacity(1024);
    let data = Data {
        year: 2022,
        month: NonZeroU8::new(2).unwrap(),
        day: 24,
        is_special: true,
        cache: Cache(Some(-123)),
        option_none: None,
        option_some: Some(()),
        array: [1337; 16],
        char: '2',
    };

    data.write_le(&mut buffer)?;
    let data_copy = Data::read::<_, LittleEndian>(&mut buffer.as_slice())?;

    assert_eq!(data, data_copy);

    Ok(())
}

#[test]
fn error_end_of_buffer() {
    let buffer = Vec::new();
    let result = u8::read_ne(&mut buffer.as_slice());
    assert_matches!(result, Err(ReadError::EndOfBuffer));
}

//#[test]
//fn error_incorrect_length() {
//    let mut buffer = Vec::new();
//    buffer.extend_from_slice(&[0u8]);
//
//    let result = <[u8; 2]>::read_ne(&mut buffer.as_slice());
//    assert_matches!(
//        result,
//        Err(ReadError::IncorrectLength {
//            expected: 2,
//            actual: 1
//        })
//    );
//}

#[test]
fn error_value_cannot_be_zero() {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&0i16.to_ne_bytes());

    let result = NonZeroI16::read_ne(&mut buffer.as_slice());
    assert_matches!(result, Err(ReadError::ZeroValue));
}
