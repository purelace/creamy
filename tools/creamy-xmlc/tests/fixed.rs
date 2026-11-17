use creamy_xmlc::{
    SemanticModel,
    symbols::{FieldSymbol, FieldType, T_U64_ID, Type},
};
use pretty_assertions::assert_eq;

const CONTENT: &str = include_str!("samples/fixed/invalid_field_range.cmp");
const CONTENT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/samples/fixed/invalid_field_range.cmp"
);

#[test]
fn invalid_field_range() {
    let result = SemanticModel::compile(CONTENT);
    let protocol = match result {
        Ok(protocol) => protocol,
        Err(d) => {
            d.print(CONTENT_PATH, CONTENT);
            panic!();
        }
    };

    let types = protocol.types();
    let ty = types
        .iter()
        .find(|t| t.ident() == protocol.pool().get_id("SpanId"))
        .expect("Type 'SpanId' not found");

    let Type::Struct(s) = ty else {
        panic!("'SpanId' is not a struct ");
    };

    let fields = s.fields().to_vec();

    assert_eq!(
        fields,
        [FieldSymbol::new(
            protocol.pool().get_id("value"),
            FieldType::Type(T_U64_ID),
            false,
        )],
    );
}
