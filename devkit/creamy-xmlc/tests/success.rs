use std::io::Cursor;

use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::StringPool;
use creamy_xmlc::{ProtocolDefinition, StringPoolResolver, compile};
use semver::Version;

const SUCCESS_TEST: &str = include_str!("success.xml");

#[test]
fn success() {
    let mut pool = StringPool::default();
    let protocol = compile(&mut pool, SUCCESS_TEST).unwrap();
    assert_eq!(protocol.name().resolve(&pool), "test");
    assert_eq!(protocol.version(), &Version::new(0, 0, 1));
    assert_eq!(protocol.table().type_count(), 19); //Builtin (12) + Custom (7)
    let first = &protocol.table().types()[12];
    assert_eq!(first.ident().resolve(&pool), "TypeOptions");

    let second = &protocol.table().types()[13];
    assert_eq!(second.ident().resolve(&pool), "TypeMeta");
}

#[test]
fn serialize_and_deserialize() {
    let mut pool = StringPool::default();

    match compile(&mut pool, SUCCESS_TEST) {
        Ok(original) => {
            let mut buffer = Vec::new();
            let mut writer = Cursor::new(&mut buffer);
            original.write_le(&mut writer).expect("Failed to serialize");

            let mut reader = Cursor::new(&buffer);
            let deserialized = ProtocolDefinition::read_le(&mut reader)
                .expect("Failed to deserialize from binary");

            assert_eq!(
                original, deserialized,
                "Protocol changed after round-trip serialization"
            );

            assert_eq!(reader.position(), buffer.len() as u64);
        }
        Err(diag) => {
            diag.print(SUCCESS_TEST);
            panic!();
        }
    }
}
