mod content;
mod lightweight;
mod stream_message;

use std::{borrow::Cow, io::Write};

use creamy_libgen::{
    EnrichedStreamMessageSymbol, SymbolIterator,
    proxy::{EnrichedFieldSymbol, EnrichedStreamFieldSymbol},
};
use heck::ToSnakeCase;

use self::{content::generate_stream_content, stream_message::generate_stream_trait_impl};
use crate::{
    Args, ExternCrateAlloc, extend_derive, generate_message_trait_impl,
    generate_typed_message_trait_impl,
    generator::{
        Access, Argument, Body, BodyLine, Field, Function, Impl, Module, Pass, Repr, Struct,
        StructContent,
    },
    message_header,
    utils::generate_message_consts,
};

fn generate_stream_module<'s, F, C, W: Write + 's>(
    args: &Args,
    symbol: &EnrichedStreamMessageSymbol<'s, F, C>,
) -> (Module<'s, W>, String)
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
{
    let mut module = Module::new(symbol.name.to_snake_case());
    module.access = Access::Pub;

    let stream_trait = generate_stream_trait_impl(
        args,
        symbol.timeout,
        symbol.name,
        format!("{}::{}Content", module.name.clone(), symbol.name),
        &format!("{}::{}Lightweight", module.name.clone(), symbol.name),
    );

    module.other.push(Box::new(ExternCrateAlloc));
    module.other.push(Box::new(
        generate_stream_content(args, symbol.name, symbol.content_fields.clone())
            .to_string()
            .unwrap(),
    ));

    let tokens =
        lightweight::generate_lightweight_content(args, symbol.name, symbol.content_fields.clone());
    module.other.push(Box::new(tokens.to_string().unwrap()));

    (module, stream_trait)
}

pub(super) fn generate_stream_message<'s, W: Write + 's, F, C>(
    symbol: &EnrichedStreamMessageSymbol<'s, F, C>,
    args: &Args,
) -> (Struct<'s, W>, Module<'s, W>)
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
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
                is_extern: false,
                name: Cow::Borrowed("with_discriminant"),
                generics: vec![],
                self_pass: Some(Pass::Mut),
                args: vec![Argument::new("value", args.stream_type_path(), Pass::Move)],
                ret: Some(Cow::Borrowed("&mut Self")),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: Cow::Borrowed("self.meta &= 0b_1111_1110;"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("let value = unsafe {"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Owned(format!(
                                "core::mem::transmute::<{}, u8>(value)",
                                args.stream_type_path(),
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
                inline: false,
            },
            Function {
                access: Access::Pub,
                is_const: true,
                is_extern: false,
                name: Cow::Borrowed("with_stream_id"),
                generics: vec![],
                self_pass: Some(Pass::Mut),
                args: vec![Argument::new("value", args.stream_id_path(), Pass::Move)],
                ret: Some(Cow::Borrowed("&mut Self")),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: Cow::Borrowed("self.meta &= 0b_0000_0001;"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("self.meta |= value.value() << 1;"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("self"),
                            depth: 0,
                        },
                    ],
                },

                inline: false,
            },
        ],

        custom: generate_message_consts(args, symbol.group, symbol.kind, symbol.dispatch_value),
    });

    struct_
        .trait_impls
        .push(generate_typed_message_trait_impl(args, symbol.name));

    struct_
        .trait_impls
        .push(generate_message_trait_impl(args, symbol.name));

    let (module, stream_trait) = generate_stream_module(args, symbol);
    struct_.custom.push(Box::new(stream_trait));
    (struct_, module)
}
