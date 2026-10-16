use creamy_binary::{BinaryReader, BinaryWriter};

#[test]
fn vec_serialization() -> Result<(), Box<dyn core::error::Error>> {
    let vec = vec![1u16; 256];
    let mut buffer = Vec::with_capacity(512);

    vec.write_ne(&mut buffer)?;

    // u32 + 256 * size_of::<u16>() = 516
    // u32 - length
    assert_eq!(buffer.len(), 516);

    let vec_copy = Vec::read_ne(&mut buffer.as_slice())?;

    assert_eq!(vec, vec_copy);

    Ok(())
}
