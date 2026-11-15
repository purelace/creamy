use creamy_protocol_model::error::SemanticError;

use self::common::zero_span;

mod common;

const ERROR_SAMPLE: &str = include_str!("samples/bitset_errors.cmp");

#[test]
fn errors() {
    let errors = common::compile(ERROR_SAMPLE).unwrap_err();
    assert_diag!(
        errors,
        vec![
            SemanticError::ForbiddenType { span: zero_span() }.into(),
            SemanticError::ForbiddenType { span: zero_span() }.into(),
            SemanticError::ZeroBitsetValueWidth { span: zero_span() }.into(),
            SemanticError::BitsetValueExceedsRepr {
                actual: 24,
                repr: "i8".into(),
                max_possible: 8,
                span: zero_span(),
            }
            .into(),
            SemanticError::BitsetValueExceedsRepr {
                actual: 1024,
                repr: "i128".into(),
                max_possible: 128,
                span: zero_span(),
            }
            .into(),
            SemanticError::InvalidSize { actual: 64 }.into(),
        ],
        ERROR_SAMPLE
    );
}
