use std::{assert_matches, io::Cursor};

use binrw::{BinRead, BinWrite, Endian};
use creamy_utils::strpool::{StringPool, StringPoolIntern, StringPoolResolver};

#[test]
#[allow(
    clippy::needless_borrow,
    reason = "StringPoolResolver trait extenstion testing"
)]
fn serialize_deserialize() -> Result<(), Box<dyn core::error::Error>> {
    let mut pool = StringPool::default();
    let value0 = "value0".intern(&mut pool);
    let value1 = pool.get_id_or_add("value1");
    let value2 = String::from("value2").intern(&mut pool);

    assert_eq!(value0.value(), 0);
    assert_eq!(value1.value(), 1);
    assert_eq!(value2.value(), 2);

    let mut bytes = Vec::with_capacity(1024);

    let mut writer = Cursor::new(&mut bytes);
    pool.write_options(&mut writer, Endian::Little, ())?;

    let pool = StringPool::read_options(&mut Cursor::new(&mut bytes), Endian::Little, ())?;
    assert_eq!(value0.resolve(&pool), "value0");
    assert_eq!((&&value1).resolve(&pool), "value1");
    assert_eq!(value2.resolve(&pool), "value2");

    Ok(())
}

#[test]
fn read_errors() {
    let mut vec = vec![];
    let result = StringPool::read_options(&mut Cursor::new(&mut vec), Endian::Little, ());

    // Could not parse the number of elements
    assert_matches!(result, Err(binrw::Error::Backtrace(_)));

    // Push the number of elements in bytes (1: u16)
    vec.extend_from_slice(&(1u16.to_le_bytes()));

    // Cound not parse the length of string
    let result = StringPool::read_options(&mut Cursor::new(&mut vec), Endian::Little, ());
    assert_matches!(result, Err(binrw::Error::Backtrace(_)));

    // Push the length of string (7: u32)
    vec.extend_from_slice(&7u32.to_le_bytes());

    // push the unicode string with forbidden character (0xff)
    vec.extend_from_slice(&[0x48, 0x65, 0x6c, 0x6c, 0x6f, 0xff, 0x21]);

    let result = StringPool::read_options(&mut Cursor::new(&mut vec), Endian::Little, ());
    assert_matches!(result, Err(binrw::Error::Backtrace(_)));

    vec.clear();

    // Push the broken string
    vec.extend_from_slice(&(1u16.to_le_bytes()));
    vec.extend_from_slice(&[0, 0, 7]);
    vec.extend_from_slice(&[1, 2, 3]);
    let result = StringPool::read_options(&mut Cursor::new(&mut vec), Endian::Little, ());
    assert_matches!(result, Err(binrw::Error::Backtrace(_)));
}
