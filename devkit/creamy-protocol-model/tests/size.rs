use creamy_binary::{BinaryReader, BinaryWriter};
use creamy_protocol_model::{Size, error::SemanticError};

#[test]
fn validate() {
    matches!(
        Size::try_new(0),
        Err(SemanticError::InvalidSize { actual: 0 })
    );

    matches!(
        Size::try_new(100),
        Err(SemanticError::InvalidSize { actual: 100 })
    );

    let result = Size::try_new(28);
    assert!(result.is_ok_and(|v| v.value() == 28));

    let result = Size::try_new(10);
    assert!(result.is_ok_and(|v| v.value() == 10));
}

//#[test]
//fn serialization() -> Result<(), Box<dyn core::error::Error>> {
//    let mut buffer = vec![];
//    let size = Size::try_new(10)?;
//
//    size.write_ne(&mut buffer)?;
//
//    let result = Size::read_ne(&mut buffer.as_slice())?;
//    assert_eq!(size, result);
//
//    Ok(())
//}
