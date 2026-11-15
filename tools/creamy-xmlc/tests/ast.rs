mod common;
mod generator;

use creamy_protocol_model::{constraints::MAX_MESSAGES_PER_PROTOCOL, error::SemanticError};
use creamy_xmlc::error::SyntaxError;

use crate::{
    common::{compile, zero_span},
    generator::{XMLGenerator, XMLGeneratorBuilder},
};

const SAMPLE: &str = include_str!("samples/ast_errors.cmp");

#[test]
fn ast_errors() {
    assert_diag!(
        compile(SAMPLE).unwrap_err(),
        vec![SyntaxError::UnexpectedToken { expected: '=' }.into()],
        SAMPLE
    );
}

//fn assert_constraint(generator: XMLGenerator, err: SemanticError) {
//    let content = get_xml("0.0.1", "Private", &generator.collect::<String>());
//    assert_diag!(compile(&content).unwrap_err(), vec![err.into()], &content);
//}
//
//#[test]
//fn too_many_messages() {
//    let generator = XMLGeneratorBuilder::default()
//        .messages(MAX_MESSAGES_PER_PROTOCOL + 1)
//        .fields_per_message(1)
//        .build();
//
//    assert_constraint(generator, SemanticError::TooManyMessages);
//}
