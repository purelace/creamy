mod common;

use creamy_xmlc::{
    VariantValue,
    error::{ProtocolError, SemanticError},
};

use crate::common::{compile, get_xml};

//#[test]
//fn double_remainder() {
//    const CONTENT: &str = r#"
//<group name="Group">
//    <message name="Message">
//        <field name="field" type="f32"/>
//        <remainder name="data"/>
//        <remainder name="data1"/>
//    </message>
//</group>
//"#;
//    assert_eq!(
//        compile(&get_xml("1337.1337", CONTENT)),
//        Err(vec![
//            AstError::UnexpectedToken("<remainder>".to_string()).into()
//        ])
//    );
//}

#[test]
fn semantic_errors() {
    const CONTENT: &str = r#"
    <group name="test" access="Public">

        <!-- Struct size limit -->
        <struct name="Sync">
            <field name="value0" type="u128"/>
            <field name="value1" type="u128"/>
            <field name="value2" type="u128"/>
            <field name="value3" type="u128"/>
            <field name="value4" type="u128"/>
            <field name="value5" type="u128"/>
            <field name="value6" type="u128"/>
            <field name="value7" type="u128"/>
            <field name="value10" type="u128"/>
            <field name="value11" type="u128"/>
            <field name="value12" type="u128"/>
            <field name="value13" type="u128"/>
            <field name="value14" type="u128"/>
            <field name="value15" type="u128"/>
        </struct>

        <!-- Message size limit -->
        <message kind="0" name="Signal">
            <field name="value0" type="u8"/>
            <field name="value1" type="u16"/>
            <field name="value2" type="u32"/>
            <field name="value3" type="u64"/>
            <field name="value4" type="u128"/>

            <field name="value5" type="i8"/>
            <field name="value6" type="i16"/>
            <field name="value7" type="i32"/>
            <field name="value8" type="i64"/>
            <field name="value9" type="i128"/>

            <field name="value10" type="f32"/>
            <field name="value11" type="f64"/>
        </message>
    </group>

    <!-- Zero sized types -->
    <group name="zst" access="Public">
        <struct name="test0"/>
        <message kind ="0" name="test1"/>
        <enum name="test2" repr="u8"/>
    </group>

    <!-- Enum errors -->
    <group name="group0" access="Public">
        <enum name="Identifier" repr="C">
            <variant name="Valid" value="0"/>
            <variant name="Invalid" value="42"/>
        </enum>

        <enum name="OutOfRange" repr="u32">
            <variant name="Error" value="-1"/>
            <variant name="Valid" value="0"/>
            <variant name="Invalid" value="1"/>
        </enum>

    </group>
"#;
    let content = get_xml("0.0.1", CONTENT);
    let errors = compile(&content).unwrap_err();
    assert_diag!(
        errors,
        vec![
            SemanticError::InvalidSize { actual: 224 }.into(),
            SemanticError::InvalidSize { actual: 84 }.into(),
            // ZST
            SemanticError::ZeroSizedType.into(),
            SemanticError::InvalidSize { actual: 0 }.into(),
            SemanticError::InvalidSize { actual: 0 }.into(),
            // Enum errors
            SemanticError::InvalidEnumUnderlyingType.into(),
            SemanticError::EnumVariantValueOutOfRange {
                value: VariantValue::Singed(-1),
                min: 0,
                max: u64::from(u32::MAX)
            }
            .into(),
        ],
        &content
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
//TODO: ref to type in the global group
//TODO: field name like struct name
//TODO: build dependency graph before resolving

//TODO: enum underlaying type
//TODO: missing group
//TODO: enum size
