use creamy_binary::{BinaryReader, BinaryWriter};
use smol_str::SmolStr;

#[test]
fn smol_str_serialization() -> Result<(), Box<dyn core::error::Error>> {
    let mut buffer = Vec::with_capacity(1024);
    let string = SmolStr::new("Hello, world!");

    string.write_be(&mut buffer)?;

    let string_copy = SmolStr::read_be(&mut buffer.as_slice())?;

    assert_eq!(string, string_copy);
    Ok(())
}
