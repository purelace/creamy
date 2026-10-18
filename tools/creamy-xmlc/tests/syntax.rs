mod common;

use creamy_xmlc::{
    Diagnostics,
    error::{ProtocolError, SyntaxError},
};

use crate::common::{compile, get_xml, zero_span};

#[test]
fn xml() {
    let content = get_xml("0.1", "<xml>");
    let errors = compile(&content).unwrap_err();
    assert!(matches!(
        errors[0],
        ProtocolError::SyntaxError(SyntaxError::Xml { span: _, error: _ })
    ));

    Diagnostics::from(errors).print(&content);
}

#[test]
fn invalid_version_format() {
    let content = get_xml("0.0.0.0", "");
    let errors = compile(&content).unwrap_err();

    assert_diag!(
        errors,
        vec![SyntaxError::InvalidVersionFormat { span: zero_span() }.into()],
        &content,
    );
}

const fn missing_attr_error(tag: &'static str, attr: &'static str) -> ProtocolError {
    ProtocolError::SyntaxError(SyntaxError::MissingAttribute {
        tag,
        attr,
        span: zero_span(),
    })
}

const CONTENT: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol>


<!-- InvalidAccess, UnknownTag -->
<group name="access" access="777">
    <error>
    </error>
</group>


<!-- Missing attributes -->
<group>
    <message kind="0" direction="Incoming">
        <field/>
    </message>

    <struct>
        <field name="data" type="u8"/>
    </struct>

    <enum>
        <variant/>
        <variant name="Foo" value="10"/>
    </enum>
</group>

<group name="test" access="Private">
    <message kind="0" name="test" direction="Duplex">
        <field name="field0" type="[u8; should_be_a_number]"/>
        <field name="field1" type="[u8;28"/>
        <field name="field2" type="u8;28]"/>
        <field name="field3" type="[28]"/>
        <field name="field4" type="[u64; 2]"/>
    </message>
</group>

<group name="42bratuha" access="Private">
    <message kind="0" name="penis@gmail.com" direction="Incoming">
        <field name="" type="u32"/>
    </message>

    <enum name="Identifier" repr="i64">
        <variant name="Error" value="NaN"/>
    </enum>
</group>

</protocol>
"#;

#[test]
fn other() {
    let _ = miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .with_syntax_highlighting(miette::highlighters::SyntectHighlighter::default())
                .context_lines(2)
                .build(),
        )
    }));

    let result = compile(CONTENT);
    let Err(errors) = result else {
        panic!();
    };

    assert_diag!(
        errors,
        vec![
            missing_attr_error("protocol", "name"),
            missing_attr_error("protocol", "version"),
            SyntaxError::InvalidIdentifier { span: zero_span() }.into(),
            SyntaxError::UnknownTag { span: zero_span() }.into(),
            missing_attr_error("group", "name"),
            missing_attr_error("group", "access"),
            missing_attr_error("message", "name"),
            missing_attr_error("field", "name"),
            missing_attr_error("field", "type"),
            missing_attr_error("struct", "name"),
            missing_attr_error("enum", "name"),
            missing_attr_error("enum", "repr"),
            missing_attr_error("variant", "name"),
            missing_attr_error("variant", "value"),
            SyntaxError::IntParse {
                span: zero_span(),
                error: "should_be_a_number".parse::<i32>().unwrap_err(),
            }
            .into(),
            SyntaxError::InvalidArraySyntax { span: zero_span() }.into(),
            SyntaxError::InvalidArraySyntax { span: zero_span() }.into(),
            SyntaxError::InvalidArraySyntax { span: zero_span() }.into(),
            SyntaxError::InvalidIdentifier { span: zero_span() }.into(),
            SyntaxError::InvalidIdentifier { span: zero_span() }.into(),
            SyntaxError::EmptyIdentifier { span: zero_span() }.into(),
            SyntaxError::NotANumber { span: zero_span() }.into(),
        ],
        CONTENT,
    );
}
