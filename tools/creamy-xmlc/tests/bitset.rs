mod common;

use creamy_protocol_model::error::SemanticError;

use self::common::compile;

const ERROR_SAMPLE: &str = include_str!("samples/bitset_errors.xml");

#[test]
fn errors() {
    let errors = compile(ERROR_SAMPLE).unwrap_err();
    assert_diag!(
        errors,
        vec![
            SemanticError::ForbiddenType.into(),
            SemanticError::ForbiddenType.into(),
            SemanticError::InvalidSize { actual: 136 }.into(),
        ],
        ERROR_SAMPLE
    );
}
