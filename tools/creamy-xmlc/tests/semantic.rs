mod common;

use creamy_xmlc::error::SemanticError;

use crate::common::compile;

const SAMPLE0: &str = include_str!("samples/semantic_errors/size_limit.cmp");
const SAMPLE0_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/samples/semantic_errors/size_limit.cmp"
);

#[test]
fn semantic_errors_size_limit() {
    let errors = compile(SAMPLE0).unwrap_err();
    assert_diag!(
        SAMPLE0_PATH,
        errors,
        vec![
            SemanticError::InvalidSize { actual: 224 }.into(),
            SemanticError::InvalidSize { actual: 96 }.into(),
        ],
        SAMPLE0,
    );
}

const SAMPLE1: &str = include_str!("samples/semantic_errors/zero_sized_types.cmp");
const SAMPLE1_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/samples/semantic_errors/zero_sized_types.cmp"
);

#[test]
fn semantic_errors_zero_sized_types() {
    let errors = compile(SAMPLE1).unwrap_err();
    assert_diag!(
        SAMPLE1_PATH,
        errors,
        vec![
            SemanticError::InvalidSize { actual: 0 }.into(),
            SemanticError::InvalidSize { actual: 0 }.into(),
            SemanticError::InvalidSize { actual: 0 }.into(),
            SemanticError::InvalidSize { actual: 0 }.into(),
        ],
        SAMPLE1,
    );
}

const SAMPLE2: &str = include_str!("samples/semantic_errors/enum.cmp");
const SAMPLE2_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/samples/semantic_errors/enum.cmp"
);

#[test]
fn semantic_errors_enums() {
    let errors = compile(SAMPLE2).unwrap_err();
    assert_diag!(
        SAMPLE2_PATH,
        errors,
        vec![
            // Enum errors
            SemanticError::InvalidEnumUnderlyingType.into(),
            SemanticError::EnumVariantValueOutOfRange {
                value: "-1".into(),
                min: 0,
                max: u64::from(u32::MAX)
            }
            .into(),
        ],
        SAMPLE2,
    );
}

/*
#[test]
fn missing_reference_in_struct() {
    let content = r#"
    <enum name="Identifier" repr="i16">
        <variant name="Error" value="-1"/>
        <variant name="Valid" value="0"/>
        <variant name="Invalid" value="1"/>
    </enum>

    <struct name="ToResolve">
        <field name="value" type="MissingType"/>
    </struct>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::CannotResolveTypeFieldNotFound {
            from: "ToResolve".to_string(),
            kind: "MissingType".to_string(),
        }])
    );
}

#[test]
fn self_reference_in_struct() {
    let content = r#"
    <struct name="ToResolve">
        <field name="value" type="ToResolve"/>
    </struct>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::SelfReference("ToResolve".to_string())])
    );
}

#[test]
fn self_reference_in_message() {
    let content = r#"
    <message name="ToResolve">
        <field name="value" type="ToResolve"/>
    </message>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::SelfReference("ToResolve".to_string())])
    );
}

#[test]
fn missing_reference_in_message() {
    let content = r#"
    <message name="ToResolve">
        <field name="value" type="MissingReference"/>
    </message>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::CannotResolveTypeFieldNotFound {
            from: "ToResolve".to_string(),
            kind: "MissingReference".to_string()
        }])
    );
}

#[test]
fn message_reference_in_message() {
    let content = r#"
    <message name="First">
        <field name="value" type="i128"/>
    </message>

    <message name="Second">
        <field name="value" type="First"/>
        <field name="code" type="Second"/>
    </message>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![
            ProtocolError::MessageReference("Second".to_string()),
            //ProtocolError::SelfReference("Second".to_string())
        ])
    );
}
*/
//TODO: unique type names (structs + enums + messages)
//TODO: message reference in messages
//TODO: message reference in structs
//TODO: self reference in messages
//TODO: loop reference
//TODO: field name like struct name
//TODO: build dependency graph before resolving

//TODO: enum underlaying type
//TODO: missing protocol
//TODO: enum size
