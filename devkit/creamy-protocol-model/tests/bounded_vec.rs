use creamy_binary::{BinaryReader, BinaryWriter};
use creamy_protocol_model::{
    BoundedVec, VariantsRange, VectorElement,
    symbols::{EnumSymbol, PrimitiveRepr},
};
use creamy_utils::strpool::StringId;

const SAMPLE: EnumSymbol = EnumSymbol::new(
    StringId::new(1337),
    PrimitiveRepr::U16,
    VariantsRange::new(0, 10),
);

fn create_vec() -> BoundedVec<EnumSymbol> {
    let mut vec = BoundedVec::<EnumSymbol>::new();
    for _ in 0..EnumSymbol::MAX_SIZE {
        assert!(vec.push(SAMPLE));
    }

    vec
}

#[test]
fn simple_operations() {
    let mut vec = create_vec();
    assert_eq!(vec.len() as usize, EnumSymbol::MAX_SIZE);
    assert!(!vec.push(SAMPLE));
}

#[test]
fn serialization() -> Result<(), Box<dyn core::error::Error>> {
    let mut buffer = Vec::new();
    let vec = create_vec();

    vec.write_ne(&mut buffer)?;
    let result = BoundedVec::<EnumSymbol>::read_ne(&mut buffer.as_slice())?;

    assert_eq!(vec, result);
    Ok(())
}
