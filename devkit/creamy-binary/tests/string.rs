use core::assert_matches;

use creamy_binary::{BinaryReader, BinaryWriter, CustomError};
use creamy_binary_derive::BinWrite;

#[test]
fn string_serialization() -> Result<(), Box<dyn core::error::Error>> {
    let string = String::from("Hello, World!");
    let mut buffer = Vec::with_capacity(512);

    string.write_ne(&mut buffer)?;

    let string_copy = String::read_ne(&mut buffer.as_slice())?;

    assert_eq!(string, string_copy);

    Ok(())
}

#[derive(BinWrite)]
struct Sample<'a> {
    string: &'a str,
}

#[test]
fn str_serialization() -> Result<(), Box<dyn core::error::Error>> {
    let sample = Sample {
        string: "Hello, World!",
    };

    let mut buffer = Vec::with_capacity(512);
    sample.write_be(&mut buffer)?;

    let result = String::read_be(&mut buffer.as_slice())?;
    assert_eq!(result, "Hello, World!");
    Ok(())
}

#[test]
fn error() {
    let string = vec![0x01, 0x00, 0x00, 0x00, 0xff];
    let result = String::read_le(&mut string.as_slice());

    assert_matches!(
        result,
        Err(creamy_binary::ReadError::Custom(CustomError { .. }))
    );
}
