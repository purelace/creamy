//#[test]
//#[ignore = "failed"]
//fn paddings() {
//    let content = r#"
//<message name="Paddings">
//    <field name="count" type="u64"/>
//    <field name="len" type="i8"/>
//    <field name="size" type="i64"/>
//</message>
//"#;
//    let mut pool = StringPool::default();
//    let def = compile(&mut pool, &get_xml(content)).unwrap();
//    let fields = def.fields_slice(def.messages()[0].fields());
//    assert_eq!(ProtocolDefinition::get_struct_paddings(fields), 4 + 7);
//}
