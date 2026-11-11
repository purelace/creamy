mod common;
mod generator;

use creamy_protocol_model::{
    constraints::{MAX_GROUPS, MAX_MESSAGES_PER_GROUP},
    error::SemanticError,
};
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
    let content = get_xml("0.0.1", &generator.collect::<String>());
    assert_diag!(compile(&content).unwrap_err(), vec![err.into()], &content);
}

#[test]
fn too_many_groups_start() {
    let generator = XMLGeneratorBuilder::default()
        .groups(MAX_GROUPS + 100)
        .messages_per_group(1)
        .fields_per_message(1)
        .build();

    assert_constraint(generator, SemanticError::TooManyGroups);
}

#[test]
fn too_many_groups_end() {
    let generator = XMLGeneratorBuilder::default()
        .groups(MAX_GROUPS + 1)
        .messages_per_group(1)
        .fields_per_message(1)
        .build();

    assert_constraint(generator, SemanticError::TooManyGroups);
}

#[test]
#[ignore = "test causes panic due to string pool overflow"]
fn too_many_messages() {
    let generator = XMLGeneratorBuilder::default()
        .groups(MAX_GROUPS)
        .messages_per_group(MAX_MESSAGES_PER_GROUP + 1) //TODO check
        .fields_per_message(1)
        .build();

    assert_constraint(generator, SemanticError::TooManyMessages);
}
