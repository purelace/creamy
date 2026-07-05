#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]

mod bit_trait_impls;
mod generator;
mod stream;

use std::borrow::Cow;

use creamy_libgen::{
    CodeGenerator, EnrichedSingleMessageSymbol, EnrichedStructSymbol,
    proxy::{EnrichedFieldSymbol, EnrichedFieldType},
};
use creamy_utils::strpool::StringPool;
use creamy_xmlc::{
    ProtocolDefinition, StringPoolResolver, TypeId,
    constraints::HEADER_BYTES,
    model::{
        definition::compute_layout,
        symbols::{
            BitsetSymbol, FieldSymbol, GroupSymbol, MessageSymbol, MessageSymbolType, StreamSymbol,
            T_I8_ID, T_I16_ID, T_I32_ID, T_I64_ID, T_I128_ID, T_U8_ID, T_U16_ID, T_U32_ID,
            T_U64_ID, T_U128_ID, Type,
        },
    },
};

use crate::{
    bit_trait_impls::{generate_bitor_impl, generate_bitxor_impl},
    generator::{
        Access, Argument, Body, BodyLine, CodeBlock, Const, DeriveList, Enum, EnumVariant, Field,
        Function, Impl, Module, Pass, Repr, Struct, StructContent, TraitImpl,
    },
    stream::generate_stream_message,
};

#[derive(Default, Clone)]
pub struct Args {
    pub eq: bool,
    pub ord: bool,
    pub hash: bool,
    pub creamy_sdk_path: String,
}

impl Args {
    pub(crate) fn typed_message_path(&self) -> String {
        format!("{}::bus::message::TypedMessage", self.creamy_sdk_path)
    }

    pub(crate) fn message_size_path(&self) -> String {
        format!("{}::bus::defines::MESSAGE_SIZE", self.creamy_sdk_path)
    }

    pub(crate) fn stream_id_path(&self) -> String {
        format!("{}::stream::StreamId", self.creamy_sdk_path)
    }

    pub(crate) fn stream_chunk_type_path(&self) -> String {
        format!("{}::stream::StreamChunkType", self.creamy_sdk_path)
    }
}

fn extend_derive(list: &mut DeriveList, args: &Args) {
    list.inner.extend_from_slice(&["Debug", "Copy", "Clone"]);

    if args.eq {
        list.inner.extend_from_slice(&["PartialEq", "Eq"]);
    }

    if args.ord {
        list.inner.extend_from_slice(&["PartialOrd", "Ord"]);
    }

    if args.hash {
        list.inner.push("Hash");
    }
}

fn message_header<'a>() -> Vec<Field<'a>> {
    vec![
        Field {
            access: Access::Pub,
            name: Cow::Borrowed("dst"),
            kind: Cow::Borrowed("u8"),
            comment: Some(Cow::Borrowed("-------- HEADER --------")),
        },
        Field {
            access: Access::Pub,
            name: Cow::Borrowed("group"),
            kind: Cow::Borrowed("u8"),
            comment: None,
        },
        Field {
            access: Access::Pub,
            name: Cow::Borrowed("src"),
            kind: Cow::Borrowed("u8"),
            comment: None,
        },
        Field {
            access: Access::Pub,
            name: Cow::Borrowed("kind"),
            kind: Cow::Borrowed("u8"),
            comment: None,
        },
    ]
}

fn generate_single_message<'a, W: std::io::Write>(
    args: &Args,
    def: &ProtocolDefinition,
    pool: &'a StringPool,
    symbol: MessageSymbol,
) -> Result<Struct<'a, W>, std::io::Error> {
    let message_name = symbol.name().resolve(pool);
    let mut struct_ = Struct::new(message_name);
    struct_.access = Access::Pub;
    struct_.repr = Repr::C { align: Some(32) };
    extend_derive(&mut struct_.derives, args);

    let mut fields = message_header();

    let slice = def.fields_slice(symbol.fields());
    let mut paddings = 0;
    let total_size = compute_layout(HEADER_BYTES, slice, |f, l| {
        if l.padding != 0 {
            fields.push(Field {
                access: Access::Pub,
                name: Cow::Owned(format!("_padding{paddings}")),
                kind: Cow::Owned(format!("[u8; {}]", l.padding)),
                comment: None,
            });
            paddings += 1;
        }

        let ty = def.table().get_type(f.type_id());
        fields.push(Field {
            access: Access::Pub,
            name: Cow::Borrowed(f.name().resolve(pool)),
            kind: Cow::Borrowed(ty.ident().resolve(pool)),
            comment: None,
        });

        Result::<(), std::io::Error>::Ok(())
    })?;

    let diff = 32 - total_size;
    if diff != 0 {
        let remainder_name = Cow::Owned(format!("_padding{paddings}"));
        //let remainder_name = if let Some(name) = symbol.remainder().into_inner() {
        //    Cow::Borrowed(name.resolve(pool))
        //} else {
        //    Cow::Owned(format!("_padding{paddings}"))
        //};
        fields.push(Field {
            access: Access::Pub,
            name: remainder_name,
            kind: Cow::Owned(format!("[u8; {diff}]")),
            comment: None,
        });
    }

    fields[4].comment = Some(Cow::Borrowed("-------- PAYLOAD --------"));

    struct_.content = StructContent::Fields(fields);
    struct_
        .trait_impls
        .push(generate_message_trait_impl(args, message_name));
    let mut impl_ = generate_builder_pattern(pool, def, message_name, slice);
    impl_.custom.push(Box::new(Const {
        access: Access::Pub,
        ident: Cow::Borrowed("KIND"),
        kind: Cow::Borrowed("u8"),
        value: Cow::Owned(symbol.kind().to_string()),
    }));

    struct_.impls.push(impl_);
    Ok(struct_)
}

fn generate_message<'a, W: std::io::Write>(
    args: &Args,
    def: &ProtocolDefinition,
    pool: &'a StringPool,
    symbol: MessageSymbolType,
) -> Result<Struct<'a, W>, std::io::Error> {
    match symbol {
        MessageSymbolType::Single(symbol) => generate_single_message(args, def, pool, symbol),
        MessageSymbolType::Stream(symbol) => generate_stream_message(args, def, pool, symbol),
    }
}

const fn is_signed_type(t: TypeId) -> bool {
    matches!(t, T_I8_ID | T_I16_ID | T_I32_ID | T_I64_ID | T_I128_ID)
}

fn get_ptr_repr(type_id: TypeId, width: u8) -> &'static str {
    match (is_signed_type(type_id), width) {
        (true, 1) => "i8",
        (true, 2) => "i16",
        (true, 3..=4) => "i32",
        (true, 5..=8) => "i64",
        (true, 9..=16) => "i128",
        (false, 1) => "u8",
        (false, 2) => "u16",
        (false, 3..=4) => "u32",
        (false, 5..=8) => "u64",
        (false, 9..=16) => "u128",
        _ => unreachable!("Unreachable width"),
    }
}

fn build_reset_mask(bytes: usize, bits: usize, shift: usize) -> String {
    assert!(shift < bytes * 8);
    let mut bit = 0;
    let mut string = String::with_capacity(bytes * 8 + bytes + 2);
    string.push('0');
    string.push('b');

    let end_pos = bytes * 8 - shift;
    let start_pos = end_pos - bits;

    for _ in 0..start_pos {
        if bit != 0 && bit % 8 == 0 {
            string.push('_');
        }
        string.push('1');
        bit += 1;
    }

    for _ in 0..bits {
        if bit != 0 && bit % 8 == 0 {
            string.push('_');
        }
        string.push('0');
        bit += 1;
    }

    for _ in 0..(bytes * 8) - bit {
        if bit != 0 && bit % 8 == 0 {
            string.push('_');
        }
        string.push('1');
        bit += 1;
    }
    string
}

fn build_mask(bytes: usize, bits: usize) -> String {
    let mut bit = 0;
    let mut string = String::with_capacity(bytes * 8 + bytes + 2);
    string.push('0');
    string.push('b');
    for _ in 0..(bytes * 8 - bits) {
        if bit != 0 && bit % 8 == 0 {
            string.push('_');
        }
        string.push('0');
        bit += 1;
    }

    for _ in 0..bits {
        if bit % 8 == 0 {
            string.push('_');
        }
        string.push('1');
        bit += 1;
    }
    string
}

fn generate_read_body<'a>(
    bytes: usize,
    width: usize,
    bits: usize,
    shift: usize,
    start_pos: usize,
    end_pos: usize,
    ptr_repr: &str,
    repr: TypeId,
    repr_str: &str,
) -> Body<'a> {
    let mut body = Body::default();
    if end_pos == 0 {
        body.with_line("let value = unsafe {")
            .with_line_depth(
                format!("(self.data.as_ptr() as *const {ptr_repr}).read_unaligned()"),
                1,
            )
            .with_line("};");
    } else {
        body.with_line(format!(
            "let target_slice = &self.data[{start_pos}..={end_pos}];"
        ))
        .with_line("let value = unsafe {")
        .with_line_depth(
            format!("(target_slice.as_ptr() as *const {ptr_repr}).read_unaligned()"),
            1,
        )
        .with_line("};");
    }

    let mask = build_mask(bytes, bits);
    match repr {
        T_I8_ID | T_I16_ID | T_I32_ID | T_I64_ID | T_I128_ID => {
            let sign_extension_shift = width * 8 - bits - shift;
            let shift = sign_extension_shift + shift;
            body.with_line(format!("let value = value << {sign_extension_shift};"))
                .with_line(format!("((value >> {shift}) & {mask}) as {repr_str}"));
        }
        T_U8_ID | T_U16_ID | T_U32_ID | T_U64_ID | T_U128_ID => {
            body.with_line(format!("((value >> {shift}) & {mask}) as {repr_str}"));
        }
        _ => unreachable!("Unreachable type"),
    }
    body
}

fn generate_set_body<'a>(
    width: usize,
    bits: usize,
    shift: usize,
    start_pos: usize,
    end_pos: usize,
    ptr_repr: &str,
) -> Body<'a> {
    let mut body = Body::default();
    let reset_mask = build_reset_mask(width, bits, shift);
    let mask = build_mask(width, bits);
    body.with_line(format!(
        "let value = (value as {ptr_repr} & {mask}) << {shift};"
    ))
    .with_line(format!(
        "let target_slice = &self.data[{start_pos}..={end_pos}];"
    ))
    .with_line("let old_value = unsafe {")
    .with_line_depth(
        format!("(target_slice.as_ptr() as *const {ptr_repr}).read_unaligned()"),
        1,
    )
    .with_line("};")
    .with_line(format!("let temp = old_value & {reset_mask};"))
    .with_line("let result = temp | value;")
    .with_line("unsafe {")
    .with_line_depth(
        format!("(target_slice.as_ptr() as *mut {ptr_repr}).write_unaligned(result);"),
        1,
    )
    .with_line("}")
    .with_line("self");
    body
}

fn generate_bitset<'a, W: std::io::Write>(
    args: &Args,
    def: &ProtocolDefinition,
    pool: &'a StringPool,
    symbol: BitsetSymbol,
) -> Struct<'a, W> {
    let mut bitset = Struct::new(symbol.name().resolve(pool));
    bitset.access = Access::Pub;
    extend_derive(&mut bitset.derives, args);

    let mut impl_block = Impl {
        target: symbol.name().resolve(pool),
        functions: vec![],
        add_assert: false,
        custom: vec![],
    };

    let mut bits = 0;
    for value in def.bvalues_slice(symbol.values()) {
        bits += value.bits();
        let bytes = value.bits().div_ceil(8);

        let end_pos = bits.div_ceil(8).saturating_sub(1);
        let start_pos = end_pos.saturating_sub(bytes);
        let width = end_pos - start_pos + 1;

        let repr_type = def.table().get_type(value.repr());
        let repr = repr_type.ident().resolve(pool);

        let ptr_repr = get_ptr_repr(value.repr(), width);
        let shift = (width * 8) - (bits - (start_pos * 8));

        impl_block.functions.push(Function {
            access: Access::Pub,
            is_const: true,
            name: Cow::Borrowed(value.name().resolve(pool)),
            self_pass: Some(Pass::Ref),
            arg: vec![],
            ret: Some(Cow::Borrowed(repr)),
            body: generate_read_body(
                bytes as usize,
                width as usize,
                value.bits() as usize,
                shift as usize,
                start_pos as usize,
                end_pos as usize,
                ptr_repr,
                value.repr(),
                repr,
            ),
        });

        impl_block.functions.push(Function {
            access: Access::Pub,
            is_const: true,
            name: Cow::Owned(format!("with_{}", value.name().resolve(pool))),
            self_pass: Some(Pass::Mut),
            arg: vec![Argument {
                name: "value",
                kind: Cow::Borrowed(repr),
                pass: Pass::Move,
            }],
            ret: Some(Cow::Borrowed("&mut Self")),
            body: generate_set_body(
                width as usize,
                value.bits() as usize,
                shift as usize,
                start_pos as usize,
                end_pos as usize,
                ptr_repr,
            ),
        });
    }

    bitset.impls.push(impl_block);

    let bytes = bits.div_ceil(8);
    bitset.content = StructContent::Fields(vec![Field {
        access: Access::Pub,
        name: Cow::Borrowed("data"),
        kind: Cow::Owned(format!("[u8; {bytes}]")),
        comment: None,
    }]);

    bitset
}

fn generate_module_from_group<'a, W: std::io::Write>(
    args: &Args,
    pool: &'a StringPool,
    def: &ProtocolDefinition,
    group: GroupSymbol,
    messages: &[MessageSymbolType],
    types: &[Type],
) -> Result<Module<'a, W>, std::io::Error> {
    let mut module = Module::new(group.name().resolve(pool));
    module.access = Access::Pub;

    for ty in types {
        match ty {
            Type::Numeric(_) | Type::Array(_) => {}
            Type::Struct(symbol) => {
                module
                    .structs
                    .push(generate_struct(args, def, pool, *symbol));
            }
            Type::Enum(symbol) => {
                module.enums.push(generate_enum(args, def, pool, *symbol));
            }
            Type::Flags(symbol) => {
                module
                    .structs
                    .push(generate_flags(args, def, pool, *symbol));
            }
            Type::Bitset(symbol) => module
                .structs
                .push(generate_bitset(args, def, pool, *symbol)),
        }
    }

    for message in messages {
        module
            .structs
            .push(generate_message(args, def, pool, *message)?);
    }

    Ok(module)
}

#[allow(clippy::too_many_lines)]
pub fn generate<W: std::io::Write>(
    writer: &mut W,
    args: Args,
    pool: &StringPool,
    definition: &ProtocolDefinition,
) -> Result<(), std::io::Error> {
    let global = definition.global();
    let mut global_module = generate_module_from_group(
        &args,
        pool,
        definition,
        global,
        definition.messages_slice(global.messages()),
        definition.types_for_group(global),
    )?;

    definition.group_iter::<std::io::Error>(|group, messages, types| {
        global_module.modules.push(generate_module_from_group(
            &args, pool, definition, group, messages, types,
        )?);
        Ok(())
    })?;

    global_module.write_to(writer, 0)
}

fn generate_builder_pattern<'a, W: std::io::Write>(
    pool: &'a StringPool,
    definition: &ProtocolDefinition,
    message: &'a str,
    fields: &[FieldSymbol],
) -> Impl<'a, W> {
    let functions = fields
        .iter()
        .map(|f| {
            let ty = definition.table().get_type(f.type_id());
            let field_name = f.name().resolve(pool);
            Function {
                access: Access::Pub,
                is_const: true,
                name: Cow::Owned(format!("with_{field_name}")),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "value",
                    kind: Cow::Borrowed(ty.ident().resolve(pool)),
                    pass: Pass::Move,
                }],
                ret: Some(Cow::Borrowed("&mut Self")),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: Cow::Owned(format!("self.{field_name} = value;")),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("self"),
                            depth: 0,
                        },
                    ],
                },
            }
        })
        .collect();
    Impl {
        target: message,
        functions,
        add_assert: true,
        custom: vec![],
    }
}

fn generate_message_trait_impl<'a>(args: &Args, message: &'a str) -> TraitImpl<'a> {
    TraitImpl {
        trait_name: Cow::Owned(args.typed_message_path()),
        target: message,
        associated_types: vec![],
        functions: vec![
            Function::default()
                .with_name("dst")
                .with_self_pass(Pass::Ref)
                .with_ret("u8")
                .with_body(Body {
                    lines: vec![BodyLine {
                        content: Cow::Borrowed("self.dst"),
                        depth: 0,
                    }],
                }),
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("with_dst"),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "dst",
                    kind: Cow::Borrowed("u8"),
                    pass: Pass::Move,
                }],
                ret: Some(Cow::Borrowed("&mut Self")),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: Cow::Borrowed("self.dst = value;"),
                            depth: 0,
                        },
                        BodyLine {
                            content: Cow::Borrowed("self"),
                            depth: 0,
                        },
                    ],
                },
            },
            Function::default()
                .with_name("src")
                .with_self_pass(Pass::Ref)
                .with_ret("u8")
                .with_body(Body {
                    lines: vec![BodyLine {
                        content: Cow::Borrowed("self.src"),
                        depth: 0,
                    }],
                }),
            Function::default()
                .with_name("group")
                .with_self_pass(Pass::Ref)
                .with_ret("u8")
                .with_body(Body {
                    lines: vec![BodyLine {
                        content: Cow::Borrowed("self.group"),
                        depth: 0,
                    }],
                }),
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("with_group"),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "group",
                    kind: Cow::Borrowed("u8"),
                    pass: Pass::Move,
                }],
                ret: Some(Cow::Borrowed("&mut Self")),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: Cow::Borrowed("self.group = value;"),
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
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("kind"),
                self_pass: Some(Pass::Ref),
                arg: vec![],
                ret: Some(Cow::Borrowed("u8")),
                body: Body {
                    lines: vec![BodyLine {
                        content: Cow::Borrowed("self.kind"),
                        depth: 0,
                    }],
                },
            },
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("with_kind"),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "kind",
                    kind: Cow::Borrowed("u8"),
                    pass: Pass::Move,
                }],
                ret: Some(Cow::Borrowed("&mut Self")),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: Cow::Borrowed("self.kind = value;"),
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
    }
}

pub struct RustGen<'a, W: std::io::Write> {
    args: Args,
    groups: Vec<Module<'a, W>>,
}

impl<'a, W: std::io::Write> CodeGenerator for RustGen<'a, W> {
    fn start_group(&mut self, group: &str) {
        self.groups.push(Module::new(group));
    }

    fn generate_single_message<'s, I>(&mut self, symbol: EnrichedSingleMessageSymbol<'s, I>)
    where
        I: Iterator<Item = EnrichedFieldSymbol<'s>>,
    {
        let mut groups = self.groups.last_mut().unwrap();

        let mut struct_ = Struct::new(symbol.name);
        struct_.access = Access::Pub;
        struct_.repr = Repr::C { align: Some(32) };
        extend_derive(&mut struct_.derives, &self.args);

        let fields = symbol
            .fields
            .map(|field| Field {
                access: Access::Pub,
                name: field.name,
                kind: match field.kind {
                    EnrichedFieldType::Type(cow) => cow,
                    EnrichedFieldType::Array { kind, len } => format!("[{kind}; {len}]").into(),
                },
                comment: None,
            })
            .collect::<Vec<_>>();

        let diff = 32 - total_size;
        if diff != 0 {
            let remainder_name = Cow::Owned(format!("_padding{paddings}"));
            //let remainder_name = if let Some(name) = symbol.remainder().into_inner() {
            //    Cow::Borrowed(name.resolve(pool))
            //} else {
            //    Cow::Owned(format!("_padding{paddings}"))
            //};
            fields.push(Field {
                access: Access::Pub,
                name: remainder_name,
                kind: Cow::Owned(format!("[u8; {diff}]")),
                comment: None,
            });
        }

        fields[4].comment = Some(Cow::Borrowed("-------- PAYLOAD --------"));

        struct_.content = StructContent::Fields(fields);
        struct_
            .trait_impls
            .push(generate_message_trait_impl(args, message_name));
        let mut impl_ = generate_builder_pattern(pool, def, message_name, slice);
        impl_.custom.push(Box::new(Const {
            access: Access::Pub,
            ident: Cow::Borrowed("KIND"),
            kind: Cow::Borrowed("u8"),
            value: Cow::Owned(symbol.kind().to_string()),
        }));

        struct_.impls.push(impl_);
        Ok(struct_)
    }

    fn generate_stream_message<'s, F, I>(
        &mut self,
        symbol: creamy_libgen::EnrichedStreamMessageSymbol<'s, F, I>,
    ) where
        F: Iterator<Item = EnrichedFieldSymbol<'s>>,
        I: Iterator<Item = EnrichedFieldSymbol<'s>>,
    {
        todo!()
    }

    fn generate_bitset(&mut self, symbol: BitsetSymbol) {
        todo!()
    }

    fn generate_flags<'s, I>(&mut self, symbol: EnrichedFlagsSymbol<'s, I>)
    where
        I: Iterator<Item = &'s str>,
    {
        todo!()
    }

    fn generate_enum<'s, I>(&mut self, symbol: EnrichedEnumSymbol<'s, I>)
    where
        I: Iterator<Item = ResolvedVariant<'s>>,
    {
        todo!()
    }

    fn generate_struct<'s, I>(&mut self, symbol: EnrichedStructSymbol<'s, I>)
    where
        I: Iterator<Item = EnrichedFieldSymbol<'s>>,
    {
        todo!()
    }

    fn end_group(&mut self) {
        todo!()
    }
}

/*
impl<'a, W: std::io::Write> CodeGenerator for RustGen<'a, W> {
    fn start_group(&mut self, name: &str) {
    }

    fn generate_single_message<'s, I>(
        &mut self,
        symbol: creamy_libgen::EnrichedSingleMessageSymbol<'s, I>,
    ) where
        I: Iterator<Item = ResolvedFieldSymbol<'s>>,
    {
        todo!()
    }

    fn generate_stream_message<'s, F, I>(
        &mut self,
        symbol: creamy_libgen::EnrichedStreamMessageSymbol<'s, F, I>,
    ) where
        F: Iterator<Item = ResolvedFieldSymbol<'s>>,
        I: Iterator<Item = ResolvedFieldSymbol<'s>>,
    {
        todo!()
    }

    fn generate_bitset(&mut self, symbol: BitsetSymbol) {
        todo!()
    }

    fn generate_flags(&mut self, symbol: EnrichedFlagsSymbol<'_>) {
        let mut flags = Struct::new(symbol.name);
        flags.access = Access::Pub;
        flags.repr = Repr::Transparent;
        extend_derive(&mut flags.derives, &self.args);

        let kind = match symbol.options.item_count() {
            1..=8 => "u8",
            9..=16 => "u16",
            17..=32 => "u32",
            33..=64 => "u64",
            65..=128 => "u128",
            other => unreachable!("Unreachable length: {other}"),
        };
        flags.content = StructContent::Tuple(vec![Cow::Borrowed(kind)]);

        let custom = symbol
            .options
            .enumerate()
            .map(|(i, s)| {
                let shift = i + 1;
                Box::new(Const {
                    access: Access::Pub,
                    ident: Cow::Borrowed(s),
                    kind: Cow::Borrowed("Self"),
                    value: Cow::Owned(format!("Self(1 << {shift})")),
                }) as Box<dyn CodeBlock<W> + 'a>
            })
            .collect::<Vec<_>>();

        flags.impls.push(Impl {
            target: symbol.name,
            functions: vec![],
            add_assert: false,
            custom,
        });

        flags
            .trait_impls
            .push(generate_bitor_impl(symbol.name, kind));
        flags
            .trait_impls
            .push(generate_bitxor_impl(symbol.name, kind));

        self.group.structs.push(flags);
    }

    fn generate_enum(&mut self, symbol: EnrichedEnumSymbol<'s>) {
        let mut enum_ = Enum::new(symbol.name);
        extend_derive(&mut enum_.derives, &self.args);
        enum_.access = Access::Pub;
        enum_.variants = symbol
            .variants
            .map(|v| EnumVariant {
                name: v.name,
                value: v.value,
            })
            .collect();

        self.group.enums.push(enum_);
    }

    fn generate_struct(&mut self, symbol: EnrichedStructSymbol<'s>) {
        let mut struct_ = Struct::new(symbol.name);
        extend_derive(&mut struct_.derives, &self.args);
        struct_.access = Access::Pub;
        struct_.content = StructContent::Fields(
            symbol
                .fields
                .map(|symbol| Field {
                    access: Access::Pub,
                    name: symbol.name,
                    kind: {
                        match symbol.kind {
                            EnrichedFieldType::Type(name) => Cow::Borrowed(name),
                            EnrichedFieldType::Array { kind, len } => {
                                Cow::Owned(format!("[{kind}; {len}]"))
                            }
                        }
                    },
                    comment: None,
                })
                .collect(),
        );

        self.group.structs.push(struct_);
    }

    fn end_group(&mut self) {}
}
*/
#[cfg(test)]
mod tests {
    use creamy_utils::strpool::StringPool;
    use creamy_xmlc::compile;

    use crate::{Args, generate};

    #[test]
    fn test() {
        let content =
            std::fs::read_to_string("/mnt/ssd/fusionwm/creamy/devkit/creamy-sdk/system.xml")
                .unwrap();
        //"/mnt/ssd/fusionwm/creamy/devkit/creamy-xmlc/tests/success.xml",
        let mut pool = StringPool::default();
        let protocol = compile(&mut pool, &content).unwrap();

        let mut bytes = Vec::with_capacity(8192 * 2);
        generate(&mut bytes, Args::default(), &pool, &protocol).unwrap();
        let string = String::from_utf8(bytes).unwrap();
        println!("{string}");
    }
}
