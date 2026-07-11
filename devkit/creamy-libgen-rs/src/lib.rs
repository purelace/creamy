#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]

pub mod creamy_libgen {
    pub use creamy_libgen::*;
}

mod bit_trait_impls;
mod builder;
mod generator;
pub mod script;
mod stream;
mod utils;

use std::{borrow::Cow, fmt::Write as FmtWrite, io::Write as IoWrite};

use creamy_libgen::{
    CodeGenerator, EnrichedSingleMessageSymbol, EnrichedStreamMessageSymbol, EnrichedStructSymbol,
    GenResult, Path, SymbolIterator,
    proxy::{
        EnrichedBitsetSymbol, EnrichedBitsetValueSymbol, EnrichedEnumSymbol, EnrichedFieldSymbol,
        EnrichedFieldType, EnrichedFlagsSymbol, EnrichedVariantSymbol, FlagUnderlyingType,
    },
};
use creamy_xmlc::model::symbols::PrimitiveRepr;
use heck::ToSnakeCase;

use self::{
    builder::generate_builder_pattern,
    generator::{FunctionDefinition, Trait},
    utils::{generate_const_size_assert, generate_message_consts},
};
use crate::{
    bit_trait_impls::{generate_bitor_impl, generate_bitxor_impl},
    generator::{
        Access, Argument, Body, BodyLine, CodeBlock, Const, DeriveList, Enum, EnumVariant, Field,
        Function, Impl, Module, Pass, Repr, Struct, StructContent, TraitImpl,
    },
};

#[derive(Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    pub eq: bool,
    pub ord: bool,
    pub hash: bool,
    pub debug_asserts: bool,
    pub creamy_sdk_path: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            eq: true,
            ord: true,
            hash: true,
            debug_asserts: true,
            creamy_sdk_path: "creamy_sdk".into(),
        }
    }
}

impl Args {
    #[must_use]
    pub fn with_creamy_sdk_path(mut self, path: impl Into<String>) -> Self {
        self.creamy_sdk_path = path.into();
        self
    }

    pub(crate) fn typed_message_trait_path(&self) -> String {
        format!("{}::bus::message::TypedMessage", self.creamy_sdk_path)
    }

    pub(crate) fn untyped_message_path(&self) -> String {
        format!("{}::bus::UntypedMessage", self.creamy_sdk_path)
    }

    pub(crate) fn message_size_path(&self) -> String {
        format!("{}::bus::defines::MESSAGE_SIZE", self.creamy_sdk_path)
    }

    pub(crate) fn custom_message_handler_trait_path(&self) -> String {
        format!("{}::api::CustomHandler", self.creamy_sdk_path)
    }

    pub(crate) fn stream_id_path(&self) -> String {
        format!("{}::stream::StreamId", self.creamy_sdk_path)
    }

    pub(crate) fn stream_chunk_type_path(&self) -> String {
        format!("{}::stream::StreamChunkType", self.creamy_sdk_path)
    }

    pub(crate) fn stream_head_trait_path(&self) -> String {
        format!("{}::stream::StreamHead", self.creamy_sdk_path)
    }

    pub(crate) fn stream_payload_trait_path(&self) -> String {
        format!("{}::stream::StreamPayload", self.creamy_sdk_path)
    }

    pub(crate) fn stream_tail_trait_path(&self) -> String {
        format!("{}::stream::StreamTail", self.creamy_sdk_path)
    }

    pub(crate) fn stream_message_trait_path(&self) -> String {
        format!("{}::stream::StreamMessage", self.creamy_sdk_path)
    }

    pub(crate) fn stream_max_payload_size_path(&self) -> String {
        format!("{}::stream::MAX_STREAM_PAYLOAD", self.creamy_sdk_path)
    }

    pub(crate) fn stream_data_trait_path(&self) -> String {
        format!("{}::stream::StreamData", self.creamy_sdk_path)
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

fn build_reset_mask(bytes: u8, bits: u8, shift: u8, is_signed: bool) -> String {
    let bytes = bytes as usize;
    let bits = bits as usize;
    let shift = shift as usize;

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

    if is_signed {
        string.push_str("u16 as i16");
    }
    string
}

fn build_mask(bytes: u8, bits: u8) -> String {
    let bytes = bytes as usize;
    let bits = bits as usize;

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

fn generate_target_slice_variable(body: &mut Body, start: u8, end: u8) {
    body.with_line("let target_slice = unsafe {");
    body.with_line_depth(
        format!(
            "core::slice::from_raw_parts(self.data.as_ptr().add({start_pos}), {end_pos})",
            start_pos = start,
            end_pos = end + 1, //TODO: важно обработать переполнение
        ),
        1,
    );
    body.with_line("};");
}

fn generate_read_body<'a>(symbol: EnrichedBitsetValueSymbol) -> Body<'a> {
    let mut body = Body::default();
    if symbol.end_pos == 0 {
        body.with_line("let value = unsafe {")
            .with_line_depth(
                format!(
                    "(self.data.as_ptr() as *const {backing_type}).read_unaligned()",
                    backing_type = repr_to_string(symbol.backing_type)
                ),
                1,
            )
            .with_line("};");
    } else {
        generate_target_slice_variable(&mut body, symbol.start_pos, symbol.end_pos);
        //body.with_line(format!(
        //    "let target_slice = &self.data[{start_pos}..={end_pos}];",
        //    start_pos = symbol.start_pos,
        //    end_pos = symbol.end_pos,
        //));
        body.with_line("let value = unsafe {")
            .with_line_depth(
                format!(
                    "(target_slice.as_ptr() as *const {backing_type}).read_unaligned()",
                    backing_type = repr_to_string(symbol.backing_type)
                ),
                1,
            )
            .with_line("};");
    }

    let mask = build_mask(symbol.bytes, symbol.bits);
    match symbol.repr {
        PrimitiveRepr::U8 | PrimitiveRepr::U16 | PrimitiveRepr::U32 | PrimitiveRepr::U64 => {
            body.with_line(format!(
                "((value >> {shift}) & {mask}) as {repr}",
                shift = symbol.shift,
                repr = repr_to_string(symbol.repr),
            ));
        }
        PrimitiveRepr::I8 | PrimitiveRepr::I16 | PrimitiveRepr::I32 | PrimitiveRepr::I64 => {
            let sign_extension_shift = symbol.read_window_bytes * 8 - symbol.bits - symbol.shift;
            let shift = sign_extension_shift + symbol.shift;
            body.with_line(format!("let value = value << {sign_extension_shift};"))
                .with_line(format!(
                    "((value >> {shift}) & {mask}) as {repr}",
                    repr = repr_to_string(symbol.repr)
                ));
        }
    }
    body
}

fn generate_set_body<'a>(symbol: EnrichedBitsetValueSymbol, add_debug_asserts: bool) -> Body<'a> {
    let mut body = Body::default();

    //TODO: добавить unchecked вариант
    if add_debug_asserts {
        let max_value = 2u64.pow(u32::from(symbol.bits));
        if symbol.repr.is_signed() {
            let min = -((max_value / 2).cast_signed());
            let max = (max_value / 2).cast_signed() - 1;

            let assert_body = format!(r#"value >= {min} && value <= {max}, "Value out of range""#);
            body.with_line(format!("debug_assert!({assert_body});"));
        } else {
            let assert_body = format!(r#"value < {max_value}, "Value out of range""#);
            body.with_line(format!("debug_assert!({assert_body});"));
        }
    }

    let reset_mask = build_reset_mask(
        symbol.read_window_bytes,
        symbol.bits,
        symbol.shift,
        symbol.backing_type.is_signed(),
    );
    let mask = build_mask(symbol.read_window_bytes, symbol.bits);
    body.with_line(format!(
        "let value = (value as {backing_type} & {mask}) << {shift};",
        backing_type = repr_to_string(symbol.backing_type),
        shift = symbol.shift,
    ));
    generate_target_slice_variable(&mut body, symbol.start_pos, symbol.end_pos);
    //body.with_line(format!(
    //    "let target_slice = &self.data[{start_pos}..={end_pos}];",
    //    start_pos = symbol.start_pos,
    //    end_pos = symbol.end_pos,
    //));
    body.with_line("let old_value = unsafe {")
        .with_line_depth(
            format!(
                "(target_slice.as_ptr() as *const {backing_type}).read_unaligned()",
                backing_type = repr_to_string(symbol.backing_type)
            ),
            1,
        )
        .with_line("};")
        .with_line(format!("let temp = old_value & {reset_mask};"))
        .with_line("let result = temp | value;")
        .with_line("unsafe {")
        .with_line_depth(
            format!(
                "(target_slice.as_ptr() as *mut {backing_type}).write_unaligned(result);",
                backing_type = repr_to_string(symbol.backing_type)
            ),
            1,
        )
        .with_line("}")
        .with_line("self");
    body
}

fn generate_message_trait_impl<'a>(args: &Args, message: &'a str) -> TraitImpl<'a> {
    TraitImpl {
        trait_name: Cow::Owned(args.typed_message_trait_path()),
        target: message.into(),
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
                args: vec![Argument {
                    name: "value".into(),
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
                inline: false,
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
                args: vec![Argument {
                    name: "value".into(),
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
                inline: false,
            },
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("kind"),
                self_pass: Some(Pass::Ref),
                args: vec![],
                ret: Some(Cow::Borrowed("u8")),
                body: Body {
                    lines: vec![BodyLine {
                        content: Cow::Borrowed("self.kind"),
                        depth: 0,
                    }],
                },
                inline: false,
            },
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("with_kind"),
                self_pass: Some(Pass::Mut),
                args: vec![Argument {
                    name: "value".into(),
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
                inline: false,
            },
        ],
        constants: vec![],
    }
}

const fn repr_to_string(repr: PrimitiveRepr) -> Cow<'static, str> {
    Cow::Borrowed(match repr {
        PrimitiveRepr::U8 => "u8",
        PrimitiveRepr::U16 => "u16",
        PrimitiveRepr::U32 => "u32",
        PrimitiveRepr::U64 => "u64",
        PrimitiveRepr::I8 => "i8",
        PrimitiveRepr::I16 => "i16",
        PrimitiveRepr::I32 => "i32",
        PrimitiveRepr::I64 => "i64",
    })
}

pub struct RustGen<'s, W: IoWrite> {
    args: Args,
    modules: Vec<Module<'s, W>>,
    writer: W,
}

impl<'s, W: IoWrite> RustGen<'s, W> {
    pub const fn new(args: Args, writer: W) -> Self {
        Self {
            args,
            modules: vec![],
            writer,
        }
    }

    fn push_struct(&mut self, struct_: Struct<'s, W>) {
        self.modules.last_mut().unwrap().structs.push(struct_);
    }

    fn push_enum(&mut self, enum_: Enum<'s>) {
        self.modules.last_mut().unwrap().enums.push(enum_);
    }

    fn push_module(&mut self, module: Module<'s, W>) {
        if let Some(parent) = self.modules.last_mut() {
            parent.modules.push(module);
        } else {
            self.modules.push(module);
        }
    }

    fn push_other(&mut self, other: Box<dyn CodeBlock<W> + 's>) {
        self.modules.last_mut().unwrap().other.push(other);
    }
}

impl<'s, W: IoWrite + 's> CodeGenerator<'s> for RustGen<'s, W> {
    fn start_group(&mut self, group: &'s str) {
        let mut module = Module::new(group);
        module.access = Access::Pub;
        self.modules.push(module);
    }

    fn generate_single_message<I>(&mut self, symbol: EnrichedSingleMessageSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedFieldSymbol<'s>>,
    {
        let mut struct_ = Struct::new(symbol.name);
        struct_.access = Access::Pub;
        struct_.repr = Repr::C { align: Some(32) };
        extend_derive(&mut struct_.derives, &self.args);

        let mut fields = symbol
            .fields
            .clone()
            .map(|field| Field {
                access: Access::Pub,
                name: field.name,
                kind: match field.kind {
                    EnrichedFieldType::Type { name } => path_to_string(&name).into(),
                    EnrichedFieldType::Array { kind, len } => format!("[{kind}; {len}]").into(),
                },
                comment: None,
            })
            .collect::<Vec<_>>();

        fields[4].comment = Some(Cow::Borrowed("-------- PAYLOAD --------"));

        struct_.content = StructContent::Fields(fields);
        struct_
            .trait_impls
            .push(generate_message_trait_impl(&self.args, symbol.name));
        let mut impl_ = generate_builder_pattern(symbol.fields, symbol.name);
        impl_.custom.extend(generate_message_consts(
            &self.args,
            symbol.group,
            symbol.kind,
            symbol.dispatch_value,
        ));

        struct_.impls.push(impl_);

        self.push_other(generate_const_size_assert(
            self.args.message_size_path(),
            symbol.name.to_string(),
        ));
        self.push_struct(struct_);
    }

    fn generate_stream_message<F, P, I>(&mut self, symbol: EnrichedStreamMessageSymbol<'s, F, P, I>)
    where
        F: SymbolIterator<EnrichedFieldSymbol<'s>>,
        P: SymbolIterator<EnrichedFieldSymbol<'s>>,
        I: SymbolIterator<EnrichedFieldSymbol<'s>>,
    {
        let (stream, io_module) =
            stream::generate_stream_message::<W, F, P, I>(&symbol, &self.args);

        self.push_other(generate_const_size_assert(
            self.args.message_size_path(),
            symbol.name.to_string(),
        ));
        self.push_struct(stream);
        self.push_module(io_module);
    }

    fn generate_bitset<I>(&mut self, symbol: EnrichedBitsetSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedBitsetValueSymbol<'s>>,
    {
        let mut bitset = Struct::new(symbol.name);
        bitset.access = Access::Pub;
        extend_derive(&mut bitset.derives, &self.args);

        let mut impl_block = Impl {
            target: symbol.name.into(),
            functions: vec![],
            custom: vec![],
        };

        let mut bits = 0;
        for symbol in symbol.values {
            bits += symbol.bits;

            impl_block.functions.push(Function {
                access: Access::Pub,
                is_const: true,
                name: Cow::Borrowed(symbol.name),
                self_pass: Some(Pass::Ref),
                args: vec![],
                ret: Some(repr_to_string(symbol.repr)),
                body: generate_read_body(symbol),
                inline: false,
            });

            impl_block.functions.push(Function {
                access: Access::Pub,
                is_const: true,
                name: Cow::Owned(format!("set_{}", symbol.name)),
                self_pass: Some(Pass::Mut),
                args: vec![Argument {
                    name: "value".into(),
                    kind: repr_to_string(symbol.repr),
                    pass: Pass::Move,
                }],
                ret: Some(Cow::Borrowed("&mut Self")),
                body: generate_set_body(symbol, self.args.debug_asserts),
                inline: false,
            });

            impl_block.functions.push(Function {
                access: Access::Pub,
                is_const: true,
                name: Cow::Owned(format!("with_{}", symbol.name)),
                self_pass: Some(Pass::MutMove),
                args: vec![Argument {
                    name: "value".into(),
                    kind: repr_to_string(symbol.repr),
                    pass: Pass::Move,
                }],
                ret: Some(Cow::Borrowed("Self")),
                body: Body {
                    lines: vec![
                        BodyLine {
                            content: format!("self.set_{}(value);", symbol.name).into(),
                            depth: 0,
                        },
                        BodyLine {
                            content: "self".into(),
                            depth: 0,
                        },
                    ],
                },
                inline: false,
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

        self.push_struct(bitset);
    }

    fn generate_flags<I>(&mut self, symbol: EnrichedFlagsSymbol<'s, I>)
    where
        I: Iterator<Item = &'s str>,
    {
        let mut flags = Struct::new(symbol.name);
        flags.access = Access::Pub;
        flags.repr = Repr::Transparent;
        extend_derive(&mut flags.derives, &self.args);

        let kind = match symbol.underlying_type {
            FlagUnderlyingType::U8 => "u8",
            FlagUnderlyingType::U16 => "u16",
            FlagUnderlyingType::U32 => "u32",
            FlagUnderlyingType::U64 => "u64",
            FlagUnderlyingType::U128 => "u128",
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
                }) as Box<dyn CodeBlock<W> + 's>
            })
            .collect::<Vec<_>>();

        flags.impls.push(Impl {
            target: symbol.name.into(),
            functions: vec![],
            custom,
        });

        flags
            .trait_impls
            .push(generate_bitor_impl(symbol.name, kind));

        flags
            .trait_impls
            .push(generate_bitxor_impl(symbol.name, kind));

        self.push_struct(flags);
    }

    fn generate_enum<I>(&mut self, symbol: EnrichedEnumSymbol<'s, I>)
    where
        I: Iterator<Item = EnrichedVariantSymbol<'s>>,
    {
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

        self.push_enum(enum_);
    }

    fn generate_struct<I>(&mut self, symbol: EnrichedStructSymbol<'s, I>)
    where
        I: Iterator<Item = EnrichedFieldSymbol<'s>>,
    {
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
                            EnrichedFieldType::Type { name } => path_to_string(&name).into(),
                            EnrichedFieldType::Array { kind, len } => {
                                Cow::Owned(format!("[{kind}; {len}]"))
                            }
                        }
                    },
                    comment: None,
                })
                .collect(),
        );

        self.push_struct(struct_);
    }

    fn end_group(&mut self) {
        if self.modules.len() <= 1 {
            return;
        }

        let group = self.modules.pop().unwrap();
        self.modules.last_mut().unwrap().modules.push(group);
    }

    fn generate_dispatcher<I>(&mut self, messages: I)
    where
        I: Iterator<Item = ::creamy_libgen::Path>,
    {
        //let mut branches = vec![];
        let mut match_string = "match dispatch_value {".to_string();

        let mut t = Trait::new("MessageHandler");
        t.bound = Some(self.args.custom_message_handler_trait_path().into());
        //t.types = vec![TraitAssociatedType {
        //    name: "Next".into(),
        //    bound: self.args.custom_message_handler_trait_path().into(),
        //}];

        for path in messages {
            let postfix = match &path {
                Path::Global { name } => name.to_snake_case(),
                Path::Absolute { components } => components.last().unwrap().to_snake_case(),
            };
            let function_name = format!("handle_{postfix}");
            let mut function = FunctionDefinition::new(function_name.clone());
            function.set_self(Pass::Mut);
            function.add_argument(Argument {
                name: "message".into(),
                kind: path_to_string(&path).into(),
                pass: Pass::Move,
            });
            t.add_function(function);

            let _ = write!(
                match_string,
                r"
                {path}::DISPATCH_VALUE => {{
                    handler.{function_name}(message.cast());
                }},
                ",
                path = path_to_string(&path)
            );
        }

        match_string.push_str("_ => handler.handle_unknown_message(dispatch_value, message) }");

        let mut last_function = FunctionDefinition::new("handle_unknown_message");
        last_function.set_self(Pass::Mut);
        last_function.add_argument(Argument {
            name: "dispatch_value".into(),
            kind: "u32".into(),
            pass: Pass::Move,
        });
        last_function.add_argument(Argument {
            name: "message".into(),
            kind: self.args.untyped_message_path().into(),
            pass: Pass::Move,
        });

        t.functions.push(last_function);

        let function = Function {
            access: Access::Pub,
            is_const: false,
            name: "dispatch_message".into(),
            self_pass: None,
            args: vec![
                Argument::new("dispatch_value", "u32", Pass::Move),
                Argument::new("message", self.args.untyped_message_path(), Pass::Move),
                Argument::new("handler", "impl MessageHandler", Pass::Mut),
            ],
            ret: None,
            body: Body {
                lines: vec![BodyLine {
                    content: match_string.into(),
                    depth: 0,
                }],
            },
            inline: true,
        };

        let mut module = Module::new("dispatcher");
        module.other.push(Box::new(function));
        module.other.push(Box::new(t));

        self.push_module(module);
    }

    fn flush(&mut self) -> GenResult {
        assert_eq!(self.modules.len(), 1);
        if let Some(global_module) = self.modules.pop() {
            global_module.write_to(&mut self.writer, 0)?;
            self.writer.flush()?;
            Ok(())
        } else {
            panic!("Missing global group");
        }
    }
}

fn path_to_string(path: &Path) -> String {
    match path {
        Path::Global { name } => name.clone(),
        Path::Absolute { components } => {
            format!("crate::{}", components.join("::"))
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use creamy_libgen::{Codegen, ProtocolLibrary};

    use crate::{Args, RustGen};

    #[test]
    fn test() {
        let manifest = r#"
[package]
id = "org.creamy.sdk"
name = "system"
version = "1.0.0"
description = "Builtin package"
repository = "https://github.com/purelace/creamy"
authors = [ "selrisu <myirisuchan@gmail.com>" ]

[core]
path = "core.wasm"
runtime = "wasm"

[protocols]
system = { version = "1.0", groups=["builtin"]}
"#;

        let mut library = ProtocolLibrary::new(manifest);
        library
            .load_all("/mnt/ssd/fusionwm/creamy/devkit/creamy-sdk/")
            .unwrap();

        let mut generator = Codegen::new(library);
        let mut rs = RustGen {
            args: Args::default(),
            modules: vec![],
            writer: Vec::with_capacity(8192 * 2),
        };

        generator.run(&mut rs).unwrap();

        let string = String::from_utf8(rs.writer).unwrap();

        println!("{string}");
    }
}
*/
