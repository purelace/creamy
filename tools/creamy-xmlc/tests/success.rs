use creamy_utils::strpool::StringPoolResolver;
use creamy_xmlc::SemanticModel;
use semver::Version;

const SAMPLE: &str = include_str!("samples/success.cmp");
const SAMPLE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/samples/success.cmp");

#[test]
fn success_path() {
    let result = SemanticModel::compile(SAMPLE);
    let protocol = match result {
        Ok(protocol) => protocol,
        Err(d) => {
            d.print(SAMPLE_PATH, SAMPLE);
            panic!();
        }
    };
    assert_eq!(protocol.name(), "test");
    assert_eq!(protocol.version(), &Version::new(0, 0, 1));
    assert_eq!(protocol.table().type_count(), 19); //Builtin (12) + Custom (7)
    let first = &protocol.table().types()[12];
    assert_eq!(first.ident().resolve(protocol.pool()), "TypeOptions");

    let second = &protocol.table().types()[13];
    assert_eq!(second.ident().resolve(protocol.pool()), "LogType");
}
