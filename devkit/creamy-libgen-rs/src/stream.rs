use std::borrow::Cow;

use creamy_utils::strpool::StringPool;
use creamy_xmlc::{ProtocolDefinition, StringPoolResolver, model::symbols::StreamSymbol};

use crate::{
    Args, extend_derive, generate_message_trait_impl,
    generator::{
        Access, Argument, Body, BodyLine, Const, Field, Function, Impl, Pass, Repr, Struct,
        StructContent,
    },
    message_header,
};

/*
pub(super) fn generate_special_impl_module<'a, W: std::io::Write>(
    args: Args,
    def: &ProtocolDefinition,
    pool: &'a StringPool,
    symbol: StreamSymbol,
) -> Module<'a, W> {
    let mut module = Module::new("to_do");
    module.access = Access::Pub;

    //let slice = def.payload_slice(symbol.payload());
    //if let Some(head) = symbol.head() {
    //    for ele in def.fields_slice(head) {}
    //}

    let mut payload = Struct::new(format!("{}Payload", symbol.name().resolve(pool)));
    payload.access = Access::Pub;
    payload.repr = Repr::C { align: None };
    //payload.content = Str
}
*/
pub(super) fn generate_stream_message<'a, W: std::io::Write>(
    args: &Args,
    def: &ProtocolDefinition,
    pool: &'a StringPool,
    symbol: StreamSymbol,
) -> Result<Struct<'a, W>, std::io::Error> {
    let message_name = symbol.name().resolve(pool);
    let mut struct_ = Struct::new(message_name);
    struct_.access = Access::Pub;
    struct_.repr = Repr::C { align: Some(32) };
    extend_derive(&mut struct_.derives, args);

    let mut fields = message_header();
    fields.extend([
        Field {
            access: Access::Pub,
            name: Cow::Borrowed("meta"),
            kind: Cow::Borrowed("u8"),
            comment: Some(Cow::Borrowed("-------- STREAM --------")),
        },
        Field {
            access: Access::Pub,
            name: Cow::Borrowed("data"),
            kind: Cow::Borrowed("[u8; 27]"),
            comment: None,
        },
    ]);

    //let slice = def.payload_slice(symbol.payload());
    struct_.content = StructContent::Fields(fields);
    struct_.impls.push(Impl {
        target: message_name,
        functions: vec![
            Function {
                access: Access::Pub,
                is_const: true,
                name: Cow::Borrowed("discriminant"),
                self_pass: Some(Pass::Ref),
                arg: vec![],
                ret: Some(Cow::Owned(args.stream_chunk_type_path())),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: Cow::Borrowed("unsafe {"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("let val = self.meta & 0b_0000_0011;"),
                            depth: 1,
                        },
                        BodyLine {
                            content: Cow::Owned(format!(
                                "core::mem::transmute::<u8, {}>(val)",
                                args.stream_chunk_type_path()
                            )),
                            depth: 1,
                        },
                        BodyLine {
                            content: Cow::Borrowed("}"),
                            depth: 0,
                        },
                    ],
                },
            },
            Function {
                access: Access::Pub,
                is_const: true,
                name: Cow::Borrowed("stream_id"),
                self_pass: Some(Pass::Ref),
                arg: vec![],
                ret: Some(Cow::Owned(args.stream_id_path())),
                body: Body {
                    lines: vec![BodyLine {
                        content: Cow::Owned(format!(
                            "{}::new((self.meta & 0b_1111_1100) >> 2)",
                            args.stream_id_path(),
                        )),
                        depth: 0,
                    }],
                },
            },
            Function {
                access: Access::Pub,
                is_const: true,
                name: Cow::Borrowed("with_discriminant"),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "value",
                    kind: Cow::Owned(args.stream_chunk_type_path()),
                    pass: Pass::Move,
                }],
                ret: Some(Cow::Borrowed("&mut Self")),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: Cow::Borrowed("self.meta &= 0b_1111_1100;"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("let value = unsafe {"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Owned(format!(
                                "core::mem::transmute::<{}, u8>(value)",
                                args.stream_chunk_type_path(),
                            )),
                            depth: 1,
                        },
                        BodyLine {
                            content: Cow::Borrowed("};"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("self.meta |= value;"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("self"),
                            depth: 0,
                        },
                    ],
                },
            },
            Function {
                access: Access::Pub,
                is_const: true,
                name: Cow::Borrowed("with_stream_id"),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "value",
                    kind: Cow::Owned(args.stream_id_path()),
                    pass: Pass::Move,
                }],
                ret: Some(Cow::Borrowed("&mut Self")),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: Cow::Borrowed("self.meta &= 0b_0000_0011;"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("self.meta |= value.value() << 2;"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("self"),
                            depth: 0,
                        },
                    ],
                },
            },
        ],
        add_assert: true,
        custom: vec![Box::new(Const {
            access: Access::Pub,
            ident: Cow::Borrowed("KIND"),
            kind: Cow::Borrowed("u8"),
            value: Cow::Owned(symbol.kind().to_string()),
        })],
    });

    struct_
        .trait_impls
        .push(generate_message_trait_impl(args, message_name));

    Ok(struct_)
}
