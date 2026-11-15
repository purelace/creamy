use creamy_protocol_model::error::SemanticError;

mod common;

const ERROR_SAMPLE: &str = include_str!("samples/bitset_errors.cmp");

#[test]
fn errors() {
    let errors = common::compile(ERROR_SAMPLE).unwrap_err();
    assert_diag!(
        errors,
        vec![
            SemanticError::ForbiddenType.into(),
            SemanticError::ForbiddenType.into(),
            SemanticError::ZeroBitsetValueWidth.into(),
            SemanticError::InvalidSize { actual: 136 }.into(),
        ],
        ERROR_SAMPLE
    );
}
