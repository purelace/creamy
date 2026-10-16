use creamy_binary::{BinaryReader, BinaryWriter};
use creamy_binary_derive::{BinRead, BinWrite};

#[derive(BinRead, BinWrite)]
#[magic(r"CMY!")]
struct Header;

#[test]
fn serialization() -> Result<(), Box<dyn core::error::Error>> {
    let mut buffer = Vec::new();
    Header.write_ne(&mut buffer)?;

    assert_eq!(buffer.len(), 4);

    Header::read_ne(&mut buffer.as_slice())?;

    Ok(())
}
