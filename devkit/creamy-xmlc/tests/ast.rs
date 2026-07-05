mod common;
mod generator;

use creamy_xmlc::{
    constraints::{MAX_ENUMS, MAX_GROUPS, MAX_MESSAGES_PER_GROUP, MAX_STRUCTS, MAX_VARIANTS},
    error::AstError,
};

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

fn assert_constraint(generator: XMLGenerator, err: AstError) {
    let content = get_xml("0.0", &generator.collect::<String>());
    assert_diag!(compile(&content).unwrap_err(), vec![err.into()], &content);
}

#[test]
fn too_many_groups_start() {
    let generator = XMLGeneratorBuilder::default()
        .groups(MAX_GROUPS + 100)
        .messages_per_group(1)
        .fields_per_message(1)
        .build();

    assert_constraint(generator, AstError::TooManyGroups);
}

#[test]
fn too_many_groups_end() {
    let generator = XMLGeneratorBuilder::default()
        .groups(MAX_GROUPS + 1)
        .messages_per_group(1)
        .fields_per_message(1)
        .build();

    assert_constraint(generator, AstError::TooManyGroups);
}

#[test]
fn message_too_many_fields() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .messages_per_group(MAX_MESSAGES_PER_GROUP)
        .fields_per_message(28)
        .build();

    assert_constraint(generator, AstError::TooManyFields);
}

#[test]
fn structs_too_many_fields() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .structs_per_group(MAX_MESSAGES_PER_GROUP)
        .fields_per_struct(28)
        .build();

    assert_constraint(generator, AstError::TooManyFields);
}

#[test]
fn too_many_messages() {
    let generator = XMLGeneratorBuilder::default()
        .groups(MAX_GROUPS)
        .messages_per_group(MAX_MESSAGES_PER_GROUP + 1) //TODO check
        .build();

    assert_constraint(generator, AstError::TooManyMessages);
}

#[test]
fn too_many_structs() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .structs_per_group(MAX_STRUCTS + 1)
        .build();

    assert_constraint(generator, AstError::TooManyStructs);
}

#[test]
fn too_many_enums() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .enums_per_group(MAX_ENUMS + 1)
        .build();

    assert_constraint(generator, AstError::TooManyEnums);
}

#[test]
fn too_many_variants() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .enums_per_group(1)
        .variants_per_enum(MAX_VARIANTS + 1)
        .build();

    assert_constraint(generator, AstError::TooManyVariants);
}
