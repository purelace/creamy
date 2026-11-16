mod common;
mod generator;

use creamy_protocol_model::{constraints::MAX_MESSAGES_PER_PROTOCOL, error::SemanticError};
use creamy_xmlc::error::SyntaxError;

use crate::{
    common::{compile, zero_span},
    generator::{CMPGeneratorBuilder, Generator},
};

const SAMPLE: &str = include_str!("samples/ast_errors.cmp");
const SAMPLE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/samples/ast_errors.cmp");

#[test]
fn ast_errors() {
    assert_diag!(
        SAMPLE_PATH,
        compile(SAMPLE).unwrap_err(),
        vec![
            SyntaxError::UnknownKeyword {
                actual: "class".into(),
                span: zero_span()
            }
            .into(),
            SyntaxError::UnexpectedToken {
                expected: "=",
                span: zero_span()
            }
            .into()
        ],
        SAMPLE
    );
}

fn assert_constraint(generator: Generator, err: SemanticError) {
    const HEADER: &str = r#"
    #name    "name"
    #version "0.0.1"
    #access  "Private"
    #schema  1
"#;
    let content = core::iter::once(HEADER.to_string())
        .chain(generator)
        .collect::<String>();
    assert_diag!(
        "",
        compile(&content).unwrap_err(),
        vec![err.into()],
        &content
    );
}

#[test]
fn too_many_messages() {
    let generator = CMPGeneratorBuilder::default()
        .messages(MAX_MESSAGES_PER_PROTOCOL + 1)
        .fields_per_message(1)
        .build();

    assert_constraint(generator, SemanticError::TooManyMessages);
}
