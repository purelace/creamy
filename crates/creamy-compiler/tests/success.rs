use creamy_compiler::{
    ProtocolCompiler, StringPoolResolver,
    model::{ResolvedType, types::Access},
};

const SUCCESS_TEST: &str = include_str!("success.xml");

#[test]
fn success() {
    let mut compiler = ProtocolCompiler::new();
    let protocol = compiler.compile(SUCCESS_TEST);
    let pool = compiler.get_pool();
    assert_eq!(protocol.name().resolve(pool), "success_test");
    assert_eq!(protocol.version(), 10);
    assert_eq!(protocol.access(), Access::MultipleWrite);
    assert_eq!(protocol.types().len(), 16); //Builtin (12) + Custom (4)
    let first = &protocol.types()[12];
    assert_eq!(first.name().resolve(pool), "Status");

    let second = &protocol.types()[13];
    assert_eq!(second.name().resolve(pool), "BucketSmall");
}
