use creamy_protocol_model::{
    Version,
    definition::{BinaryReader, BinaryWriter, ProtocolModel},
};
use creamy_utils::strpool::{StringPool, StringPoolResolver};
use creamy_xmlc::compile;

const SAMPLE: &str = include_str!("samples/success.xml");

#[test]
fn success_path() {
    let mut pool = StringPool::default();
    let result = compile(&mut pool, SAMPLE);
    let protocol = match result {
        Ok(protocol) => protocol,
        Err(d) => {
            d.print(SAMPLE);
            panic!();
        }
    };
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

    match compile(&mut pool, SAMPLE) {
        Ok(original) => {
            let mut buffer = Vec::new();
            original.write_ne(&mut buffer).expect("Failed to serialize");

            let deserialized = ProtocolModel::read_ne(&mut buffer.as_slice())
                .expect("Failed to deserialize from binary");

            assert_eq!(
                original, deserialized,
                "Protocol changed after round-trip serialization"
            );
        }
        Err(diag) => {
            diag.print(SAMPLE);
            panic!();
        }
    }
}
