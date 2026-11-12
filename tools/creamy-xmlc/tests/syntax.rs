mod common;

use creamy_xmlc::{
    Diagnostics,
    error::{ProtocolError, SyntaxError},
};

use crate::common::{compile, get_xml, zero_span};

#[test]
fn xml() {
    let content = get_xml("0.1", "Private", "<xml>");
    let errors = compile(&content).unwrap_err();
    assert!(matches!(
        errors[0],
        ProtocolError::SyntaxError(SyntaxError::Xml { span: _, error: _ })
    ));

    Diagnostics::from(errors).print(&content);
}

#[test]
fn invalid_version_format() {
    let content = get_xml("0.0.0.0", "Private", "");
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

const SAMPLE0: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<!-- InvalidAccess, UnknownTag -->
<protocol name="access" version="0.0.1+testbuild" access="777">
    <error>
    </error>
</protocol>
"#;

const SAMPLE1: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol>
    <!-- Missing attributes -->
    <message direction="Incoming">
        <field/>
    </message>

    <struct>
        <field name="data" type="u8"/>
    </struct>

    <enum>
        <variant/>
        <variant name="Foo" value="10"/>
    </enum>
</protocol>
"#;

const SAMPLE2: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol name="test" version="0.1.0" access="Private">
    <message name="test" direction="Duplex">
        <field name="field0" type="[u8; should_be_a_number]"/>
        <field name="field1" type="[u8;28"/>
        <field name="field2" type="u8;28]"/>
        <field name="field3" type="[28]"/>
        <field name="field4" type="[u64; 2]"/>
    </message>
</protocol>
"#;

const SAMPLE3: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol name="42bratuha" version="5.0.0" access="Private">
    <message name="penis@gmail.com" direction="Incoming">
        <field name="zak" type="Ruins"/>
        <field name="" type="u8"/>
    </message>

    <enum name="Identifier" repr="i64">
        <variant name="Error" value="NaN"/>
    </enum>
</protocol>
"#;

fn set_hook() {
    let _ = miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .with_syntax_highlighting(miette::highlighters::SyntectHighlighter::default())
                .context_lines(2)
                .build(),
        )
    }));
}

#[test]
fn sample0() {
    set_hook();

    let result = compile(SAMPLE0);
    let Err(errors) = result else {
        panic!();
    };

    assert_diag!(
        errors,
        vec![
            SyntaxError::InvalidIdentifier { span: zero_span() }.into(),
            SyntaxError::UnknownTag { span: zero_span() }.into(),
        ],
        SAMPLE0
    );
}

#[test]
fn sample1() {
    set_hook();

    let result = compile(SAMPLE1);
    let Err(errors) = result else {
        panic!();
    };

    assert_diag!(
        errors,
        vec![
            missing_attr_error("protocol", "name"),
            missing_attr_error("protocol", "version"),
            missing_attr_error("protocol", "access"),
            missing_attr_error("message", "name"),
            missing_attr_error("field", "name"),
            missing_attr_error("field", "type"),
            missing_attr_error("struct", "name"),
            missing_attr_error("enum", "name"),
            missing_attr_error("enum", "repr"),
            missing_attr_error("variant", "name"),
            missing_attr_error("variant", "value"),
        ],
        SAMPLE1
    );
}

#[test]
fn sample2() {
    set_hook();

    let result = compile(SAMPLE2);
    let Err(errors) = result else {
        panic!();
    };

    assert_diag!(
        errors,
        vec![
            SyntaxError::IntParse {
                span: zero_span(),
                error: "should_be_a_number".parse::<i32>().unwrap_err(),
            }
            .into(),
            SyntaxError::InvalidArraySyntax { span: zero_span() }.into(),
            SyntaxError::InvalidArraySyntax { span: zero_span() }.into(),
            SyntaxError::InvalidArraySyntax { span: zero_span() }.into(),
        ],
        SAMPLE2
    );
}

#[test]
fn sample3() {
    set_hook();

    let result = compile(SAMPLE3);
    let Err(errors) = result else {
        panic!();
    };

    assert_diag!(
        errors,
        vec![
            SyntaxError::InvalidIdentifier { span: zero_span() }.into(),
            SyntaxError::InvalidIdentifier { span: zero_span() }.into(),
            SyntaxError::EmptyIdentifier { span: zero_span() }.into(),
            SyntaxError::NotANumber { span: zero_span() }.into(),
        ],
        SAMPLE3
    );
}
