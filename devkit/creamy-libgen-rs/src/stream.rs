use std::{borrow::Cow, io::Write};

use creamy_libgen::{
    EnrichedStreamMessageSymbol, SymbolIterator,
    proxy::{EnrichedFieldSymbol, EnrichedFieldType},
};
use heck::ToSnakeCase;

use crate::{
    Args, extend_derive, generate_message_trait_impl,
    generator::{
        Access, Argument, Body, BodyLine, Const, Field, Function, Impl, Module, Pass, Repr, Struct,
        StructContent, TraitImpl, TraitImplAssociatedType,
    },
    message_header,
    utils::{generate_const_size_assert, generate_message_consts},
};

fn generate_stream_trait_impl<'s>(
    args: &Args,
    timeout: u8,
    target: impl Into<Cow<'s, str>>,
) -> TraitImpl<'s> {
    TraitImpl {
        trait_name: args.stream_message_trait_path().into(),
        target: target.into(),
        associated_types: vec![],
        functions: vec![
            Function {
                access: Access::None,
                is_const: false,
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
                access: Access::None,
                is_const: false,
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
        ],
        constants: vec![Const {
            access: Access::None,
            ident: "TIMEOUT".into(),
            kind: "u8".into(),
            value: timeout.to_string().into(),
        }],
    }
}

fn generate_part<'s, I, W: Write + 's>(
    part_name: String,
    trait_marker: String,
    fields: I,
) -> Struct<'s, W>
where
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    let mut part = Struct::new(part_name.clone());
    part.access = Access::Pub;
    part.repr = Repr::C { align: None };

    let fields = fields
        .map(|field| Field {
            access: Access::Pub,
            name: field.name,
            kind: match field.kind {
                EnrichedFieldType::Type(cow) => cow,
                EnrichedFieldType::Array { kind, len } => format!("[{kind}; {len}]").into(),
            },
            comment: None,
        })
        .collect();

    part.content = StructContent::Fields(fields);
    part.trait_impls.push(TraitImpl {
        trait_name: trait_marker.into(),
        //TODO нет смысла хранить target
        target: part_name.clone().into(),
        associated_types: vec![],
        functions: vec![],
        constants: vec![],
    });

    part
}

fn generate_stream_module<'s, F, P, I, W: Write + 's>(
    args: &Args,
    symbol: &EnrichedStreamMessageSymbol<'s, F, P, I>,
) -> (Module<'s, W>, TraitImpl<'s>)
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    P: SymbolIterator<EnrichedFieldSymbol<'s>>,
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    let mut module = Module::new(symbol.name.to_snake_case());
    module.access = Access::Pub;

    let mut stream_trait = generate_stream_trait_impl(args, symbol.timeout, symbol.name);

    if let Some(head) = symbol.head.clone() {
        let name = format!("{}Head", symbol.name);
        stream_trait.associated_types.push(TraitImplAssociatedType {
            name: "Head",
            kind: format!("{}::{}", symbol.name.to_snake_case(), name.clone()).into(),
        });
        module.other.push(generate_const_size_assert(
            args.stream_max_payload_size_path(),
            name.clone(),
        ));
        module
            .structs
            .push(generate_part(name, args.stream_head_trait_path(), head));
    } else {
        stream_trait.associated_types.push(TraitImplAssociatedType {
            name: "Head",
            kind: "()".into(),
        });
    }

    let payload_name = format!("{}Payload", symbol.name);
    stream_trait.associated_types.push(TraitImplAssociatedType {
        name: "Payload",
        kind: format!("{}::{}", symbol.name.to_snake_case(), payload_name.clone()).into(),
    });
    module.other.push(generate_const_size_assert(
        args.stream_max_payload_size_path(),
        payload_name.clone(),
    ));
    module.structs.push(generate_part(
        payload_name,
        args.stream_payload_trait_path(),
        symbol.payload.clone(),
    ));

    if let Some(tail) = symbol.tail.clone() {
        let name = format!("{}Tail", symbol.name);
        stream_trait.associated_types.push(TraitImplAssociatedType {
            name: "Tail",
            kind: format!("{}::{}", symbol.name.to_snake_case(), name.clone()).into(),
        });
        module.other.push(generate_const_size_assert(
            args.stream_max_payload_size_path(),
            name.clone(),
        ));
        module
            .structs
            .push(generate_part(name, args.stream_tail_trait_path(), tail));
    } else {
        stream_trait.associated_types.push(TraitImplAssociatedType {
            name: "Tail",
            kind: "()".into(),
        });
    }

    (module, stream_trait)
}

pub(super) fn generate_stream_message<'s, W: Write + 's, F, P, I>(
    symbol: &EnrichedStreamMessageSymbol<'s, F, P, I>,
    args: &Args,
) -> (Struct<'s, W>, Module<'s, W>)
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    P: SymbolIterator<EnrichedFieldSymbol<'s>>,
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    let mut struct_ = Struct::new(symbol.name);
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

    struct_.content = StructContent::Fields(fields);
    struct_.impls.push(Impl {
        target: symbol.name.into(),
        functions: vec![
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

        custom: generate_message_consts(args, symbol.kind, symbol.dispatch_value),
    });

    struct_
        .trait_impls
        .push(generate_message_trait_impl(args, symbol.name));

    let (module, stream_trait) = generate_stream_module(args, symbol);
    struct_.trait_impls.push(stream_trait);
    (struct_, module)
}
