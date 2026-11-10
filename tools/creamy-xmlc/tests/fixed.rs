use creamy_protocol_model::symbols::{FieldSymbol, FieldType, T_U64_ID, Type};
use creamy_utils::strpool::StringPool;
use creamy_xmlc::compile;
use pretty_assertions::assert_eq;

const CONTENT: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol name="system" version="1.0.0">
    <group name="builtin" access="Public">

        <enum name="Level" repr="u8">
            <variant name="Trace" value="0"/>
            <variant name="Debug" value="1"/>
            <variant name="Info"  value="2"/>
            <variant name="Warning" value="3"/>
            <variant name="Error" value="4"/>
        </enum>

        <bitset name="TypeMeta">
            <value name="is_valid" bits="1" repr="u8" /> <!-- TODO: bool -->
            <value name="is_const" bits="1" repr="u8" /> <!-- TODO: bool -->

            <value name="align" bits="3" repr="u16" />
            <value name="size" bits="5" repr="u16" />

            <value name="attributes_mask" bits="6" repr="u32" />
            <value name="offset_modifier" bits="4" repr="i8" />
            <value name="custom_type_id" bits="8" repr="u64" />
            <value name="version" bits="4" repr="u8" />
        </bitset>

        <flags name="Permissions">
            <option name="User"/>
            <option name="Group"/>
            <option name="Others"/>
            <option name="All"/>
        </flags>

        <message name="SetPermissions" direction="Incoming">
            <field name="permissions" type="Permissions"/>
            <field name="class" type="u16"/>
        </message>

        <stream name="Log" direction="Outgoing" timeout="60">
            <field name="level" type="Level"/>
            <field name="content" type="Array[u8]"/>
        </stream>

        <struct name="SpanId">
            <field name="value" type="u64"/>
        </struct>

    </group>
</protocol>
"#;

#[test]
fn invalid_field_range() {
    let mut pool = StringPool::default();
    let result = compile(&mut pool, CONTENT);
    let protocol = match result {
        Ok(protocol) => protocol,
        Err(d) => {
            d.print(CONTENT);
            panic!();
        }
    };

    let group = protocol
        .groups()
        .iter()
        .find(|g| g.ident() == pool.get_id("builtin"))
        .expect("Group 'builtin' not found");

    let types = protocol.types_for_group(*group);
    let ty = types
        .iter()
        .find(|t| t.ident() == pool.get_id("SpanId"))
        .expect("Type 'SpanId' not found");

    let Type::Struct(s) = ty else {
        panic!("'SpanId' is not a struct ");
    };

    let fields = protocol.fields_range(s.fields()).to_vec();

    assert_eq!(
        fields,
        [FieldSymbol::new(
            pool.get_id("value"),
            FieldType::Type(T_U64_ID),
            false,
        )],
    );
}
