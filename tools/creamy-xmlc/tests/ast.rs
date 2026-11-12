mod common;
mod generator;

use creamy_protocol_model::{constraints::MAX_MESSAGES_PER_PROTOCOL, error::SemanticError};
use creamy_xmlc::error::AstError;

use crate::{
    common::{compile, get_xml, zero_span},
    generator::{XMLGenerator, XMLGeneratorBuilder},
};

#[test]
fn ast_errors() {
    const CONTENT: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<enum name="missing" repr="i64">
    <field name="bytes" type="u16"/>
</enum>
"#;
    assert_diag!(
        compile(CONTENT).unwrap_err(),
        vec![
            AstError::MissingProtocolToken.into(),
            AstError::UnexpectedToken { span: zero_span() }.into(),
        ],
        CONTENT
    );
}

fn assert_constraint(generator: XMLGenerator, err: SemanticError) {
    let content = get_xml("0.0.1", "Private", &generator.collect::<String>());
    assert_diag!(compile(&content).unwrap_err(), vec![err.into()], &content);
}

#[test]
fn too_many_messages() {
    let generator = XMLGeneratorBuilder::default()
        .messages(MAX_MESSAGES_PER_PROTOCOL + 1)
        .fields_per_message(1)
        .build();

    assert_constraint(generator, SemanticError::TooManyMessages);
}
