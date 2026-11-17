use std::{
    collections::{HashMap, HashSet},
    num::ParseIntError,
    str::FromStr,
};

use petgraph::{
    algo::toposort,
    graph::{DiGraph, NodeIndex},
};
use semver::Version;

use crate::{
    Diagnostics, MAX_PAYLOAD,
    ast::{
        AttributeListNode, AttributeParameterListNode, AttributeParameterValueNode, BitsetNode,
        BitsetValueNode, ComplexType, DefineValueNode, DirectiveNode, EnumNode, FieldNode,
        FlagsNode, IdentifierNode, MessageNode, MessageNodeType, ModifierNode,
        NamedAttributeParameterNode, NumberNode, ObjectNode, OptionNode, StructNode, SyntaxTree,
        SyntaxTreeBuilder, TypeRefNode, VariantNode,
    },
    error::{Fallback, ProtocolErrorExt, SemanticError, SyntaxError},
    symbols::{
        ApiStatus, ArraySymbol, BitsetSymbol, BitsetValueSymbol, EnumSymbol, F32_ID, F64_ID,
        FieldSymbol, FieldType, FlagsSymbol, HeapArraySymbol, I8_ID, I16_ID, I32_ID, I64_ID,
        I128_ID, LayoutEntry, MessageSymbol, MessageSymbolType, Number, ObjectSymbol, OptionSymbol,
        PrimitiveRepr, SerializationLayout, SpecialType, StructSymbol, T_I8_ID, T_I16_ID, T_I32_ID,
        T_I64_ID, T_I128_ID, T_U8_ID, T_U16_ID, T_U32_ID, T_U64_ID, T_U128_ID, Type, U8_ID, U16_ID,
        U32_ID, U64_ID, U128_ID, VariantSymbol, get_builtin_type_name,
    },
    table::{FinishedSymbolTable, SymbolTable, TypeId, TypeMeta},
    tokenizer::Span,
    utils::{
        Access, Direction, Size,
        layout::{LayoutCalculator, LayoutItem, compute_layout},
        strpool::{StringId, StringPool, StringPoolIntern, StringPoolResolver},
    },
};

#[derive(Default)]
pub struct Scope {
    inner: HashSet<StringId>,
}

impl Scope {
    fn check_name(
        &mut self,
        diag: &mut Diagnostics,
        pool: &mut StringPool,
        ident: &IdentifierNode,
    ) -> bool {
        let id = ident.value.intern(pool);
        if self.inner.insert(id) {
            true
        } else {
            diag.report_err(SemanticError::NameDefinedMultipleTimes {
                name: ident.value.to_string(),
                span: ident.span.into(),
            });
            false
        }
    }
}

pub struct Context {
    scope: Vec<Scope>,
    diag: Diagnostics,
    pool: StringPool,
    table: SymbolTable,
    messages: Vec<MessageSymbol>,
    kind: Option<u8>,
    too_many_messages_reported: bool,
}

impl Context {
    fn new(types: u16) -> Self {
        let mut pool = StringPool::default();
        assert_eq!(pool.get_id_or_add("u8"), U8_ID);
        assert_eq!(pool.get_id_or_add("u16"), U16_ID);
        assert_eq!(pool.get_id_or_add("u32"), U32_ID);
        assert_eq!(pool.get_id_or_add("u64"), U64_ID);
        assert_eq!(pool.get_id_or_add("u128"), U128_ID);

        assert_eq!(pool.get_id_or_add("i8"), I8_ID);
        assert_eq!(pool.get_id_or_add("i16"), I16_ID);
        assert_eq!(pool.get_id_or_add("i32"), I32_ID);
        assert_eq!(pool.get_id_or_add("i64"), I64_ID);
        assert_eq!(pool.get_id_or_add("i128"), I128_ID);

        assert_eq!(pool.get_id_or_add("f32"), F32_ID);
        assert_eq!(pool.get_id_or_add("f64"), F64_ID);

        Self {
            scope: vec![Scope::default()],
            diag: Diagnostics::default(),
            pool,
            table: SymbolTable::new(types),
            messages: vec![],
            kind: Some(0),
            too_many_messages_reported: false,
        }
    }

    fn issue_kind(&mut self) -> u8 {
        if let Some(current) = self.kind {
            self.kind = current.checked_add(1);
            current
        } else {
            if !self.too_many_messages_reported {
                self.diag.report_err(SemanticError::TooManyMessages);
                self.too_many_messages_reported = true;
            }
            0
        }
    }

    fn contains_name_in_scope(&mut self, ident: &IdentifierNode) -> bool {
        let index = self.scope.len() - 1;
        !self.scope[index].check_name(&mut self.diag, &mut self.pool, ident)
    }

    fn push_scope(&mut self) {
        self.scope.push(Scope::default());
    }

    fn pop_scope(&mut self) {
        self.scope.pop();
    }
}

fn build_dependency_graph<'src>(
    pool: &mut StringPool,
    table: &SymbolTable,
    tree: &'src SyntaxTree<'src>,
) -> (HashMap<NodeIndex, usize>, DiGraph<&'src str, ()>) {
    let mut map = HashMap::new();
    let mut transform = HashMap::new();
    let mut graph = DiGraph::<&str, ()>::new();

    for (i, node) in tree.complex.iter().enumerate() {
        let index = graph.add_node(node.ident());
        map.insert(node.ident(), index);
        transform.insert(index, i);
    }

    let mut f = |root_ident: &str, fields: &[FieldNode]| {
        for node in fields {
            let ident = node.kind.ident();
            let id = ident.intern(pool);
            if table.contains_name(id) {
                return;
            }

            let a = *map.get(root_ident).unwrap();
            let b = *map.get(ident).unwrap();
            graph.add_edge(a, b, ());
        }
    };

    for root in &tree.complex {
        f(root.ident(), root.fields());
    }

    (transform, graph)
}

fn resolve_option_list(context: &mut Context, nodes: &[OptionNode<'_>]) -> Box<[OptionSymbol]> {
    context.push_scope();

    let mut options = Vec::with_capacity(nodes.len());
    for root in nodes {
        let ident = root.ident.value.intern(&mut context.pool);
        if context.contains_name_in_scope(&root.ident) {
            continue;
        }

        options.push(OptionSymbol { ident });
    }

    context.pop_scope();
    options.into_boxed_slice()
}

fn get_flags_type_metadata(diag: &mut Diagnostics, options_count: usize) -> TypeMeta {
    let bits = options_count;
    let bytes = bits.div_ceil(8);
    match bytes {
        0 => {
            diag.report_err(SemanticError::InvalidSize { actual: 0 });
            TypeMeta::S1B_A1B
        }
        1 => TypeMeta::S1B_A1B,
        2 => TypeMeta::S2B_A2B,
        3 | 4 => TypeMeta::S4B_A4B,
        5..=8 => TypeMeta::S8B_A8B,
        9..=16 => TypeMeta::S16B_A16B,
        _ => unreachable!(),
    }
}

fn resolve_flags(context: &mut Context, nodes: &[FlagsNode<'_>]) {
    for root in nodes {
        if context.contains_name_in_scope(&root.ident) {
            continue;
        }

        let options = resolve_option_list(context, &root.options);
        context.table.register_type(
            get_flags_type_metadata(&mut context.diag, root.options.len()),
            Type::Flags(FlagsSymbol {
                ident: root.ident.value.intern(&mut context.pool),
                options,
            }),
        );
    }
}

pub fn to_number(node: &NumberNode, repr: PrimitiveRepr) -> Result<Number, ParseIntError> {
    let s = node.value;
    match repr {
        PrimitiveRepr::U8 => s.parse::<u8>().map(|v| Number::Unsigned(u64::from(v))),
        PrimitiveRepr::U16 => s.parse::<u16>().map(|v| Number::Unsigned(u64::from(v))),
        PrimitiveRepr::U32 => s.parse::<u32>().map(|v| Number::Unsigned(u64::from(v))),
        PrimitiveRepr::U64 => s.parse::<u64>().map(Number::Unsigned),

        PrimitiveRepr::I8 => s.parse::<i8>().map(|v| Number::Signed(i64::from(v))),
        PrimitiveRepr::I16 => s.parse::<i16>().map(|v| Number::Signed(i64::from(v))),
        PrimitiveRepr::I32 => s.parse::<i32>().map(|v| Number::Signed(i64::from(v))),
        PrimitiveRepr::I64 => s.parse::<i64>().map(Number::Signed),
    }
}

fn resolve_variants(
    diag: &mut Diagnostics,
    pool: &mut StringPool,
    nodes: &[VariantNode<'_>],
    repr: PrimitiveRepr,
) -> Box<[VariantSymbol]> {
    let mut scope = HashSet::new();

    let mut variants = Vec::with_capacity(nodes.len());
    for root in nodes {
        match to_number(&root.value, repr) {
            Ok(value) => {
                let ident = root.ident.value.intern(pool);
                if !scope.insert(ident) {
                    diag.report_err(SemanticError::NameDefinedMultipleTimes {
                        name: root.ident.value.to_string(),
                        span: root.ident.span.into(),
                    });
                    continue;
                }

                variants.push(VariantSymbol { ident, value });
            }
            Err(_) => {
                diag.report_err(SemanticError::EnumVariantValueOutOfRange {
                    value: root.value.value.to_string(),
                    min: repr.get_min(),
                    max: repr.get_max(),
                });
            }
        }
    }

    variants.into_boxed_slice()
}

fn resolve_enums(context: &mut Context, nodes: &[EnumNode<'_>]) {
    for root in nodes {
        if context.contains_name_in_scope(&root.ident) {
            continue;
        }

        if root.variants.is_empty() {
            context
                .diag
                .report_err(SemanticError::InvalidSize { actual: 0 });
        }

        //TODO: fallback
        let repr = match PrimitiveRepr::try_from(root.modifier.ident.value) {
            Ok(repr) => repr,
            Err(err) => {
                context.diag.report_err(err);
                PrimitiveRepr::U32
            }
        };

        let symbol = EnumSymbol {
            ident: root.ident.value.intern(&mut context.pool),
            repr,
            variants: resolve_variants(&mut context.diag, &mut context.pool, &root.variants, repr),
        };
        context.table.register_type(
            symbol.meta().or_recover(&mut context.diag),
            Type::Enum(symbol),
        );
    }
}

const fn is_numeric_type(type_id: TypeId) -> bool {
    matches!(
        type_id,
        T_U8_ID
            | T_U16_ID
            | T_U32_ID
            | T_U64_ID
            | T_U128_ID
            | T_I8_ID
            | T_I16_ID
            | T_I32_ID
            | T_I64_ID
            | T_I128_ID
    )
}

fn resolve_bitset_values(
    context: &mut Context,
    nodes: &[BitsetValueNode<'_>],
) -> (u64, Box<[BitsetValueSymbol]>) {
    context.push_scope();

    let mut values = Vec::with_capacity(nodes.len());
    let mut total_bits = 0;
    for root in nodes {
        if context.contains_name_in_scope(&root.ident) {
            continue;
        }

        let width = root.kind.width.value.parse::<u64>().unwrap();

        if width == 0 {
            context
                .diag
                .report_err(SemanticError::ZeroBitsetValueWidth {
                    span: root.kind.width.span.into(),
                });
            continue;
        }

        let mut has_error = false;

        let repr = if let Some(t) = context
            .table
            .get_type_by_name(root.kind.ident.value.intern(&mut context.pool))
        {
            if is_numeric_type(t) {
                t
            } else {
                has_error = true;
                context.diag.report_err(SemanticError::ForbiddenType {
                    span: root.kind.ident.span.into(),
                });
                TypeId::fallback()
            }
        } else {
            has_error = true;
            context.diag.report_err(SemanticError::ForbiddenType {
                span: root.kind.ident.span.into(),
            });
            TypeId::fallback()
        };

        if width > repr.size().value() * 8 && !has_error {
            context
                .diag
                .report_err(SemanticError::BitsetValueExceedsRepr {
                    actual: width,
                    repr: get_builtin_type_name(repr)
                        .resolve(&context.pool)
                        .to_string(),
                    max_possible: repr.size().value() * 8,
                    span: root.kind.span.into(),
                });
            total_bits += 1;
        } else {
            total_bits += width;
        }
        values.push(BitsetValueSymbol {
            ident: root.ident.value.intern(&mut context.pool),
            repr,
            width: width as u8,
        });
    }

    let bytes = total_bits.div_ceil(8);
    let values = if bytes > MAX_PAYLOAD {
        Box::default()
    } else {
        values.into_boxed_slice()
    };

    context.pop_scope();

    (bytes, values)
}

fn resolve_bitsets(context: &mut Context, nodes: &[BitsetNode<'_>]) {
    for root in nodes {
        if context.contains_name_in_scope(&root.ident) {
            continue;
        }

        let (bytes, values) = resolve_bitset_values(context, &root.values);
        let meta = TypeMeta::try_new(bytes, 1).or_recover(&mut context.diag);
        let meta = if meta.size().value() > MAX_PAYLOAD {
            context.diag.report_err(SemanticError::InvalidSize {
                actual: meta.size().value(),
            });
            TypeMeta::fallback()
        } else {
            meta
        };
        context.table.register_type(
            meta,
            Type::Bitset(BitsetSymbol {
                ident: root.ident.value.intern(&mut context.pool),
                values,
            }),
        );
    }
}

fn resolve_field(
    diag: &mut Diagnostics,
    pool: &mut StringPool,
    table: &mut SymbolTable,
    node: &FieldNode,
    is_object: bool,
) -> FieldSymbol {
    let kind = match &node.kind {
        TypeRefNode::Identifier(n) => {
            FieldType::Type(table.get_type_by_name(n.value.intern(pool)).unwrap())
        }
        TypeRefNode::StaticArray(n) => {
            let kind = table.get_type_by_name(n.kind.value.intern(pool)).unwrap();
            let size = n.size.value.parse::<u64>().unwrap();
            let size = Size::try_new(size).or_recover(diag);
            FieldType::StaticArray(ArraySymbol::new(kind, size))
        }
        TypeRefNode::DynamicArray(n) => {
            if !is_object {
                todo!("report err")
            }

            let kind = table.get_type_by_name(n.ident().intern(pool)).unwrap();
            FieldType::HeapArray(HeapArraySymbol::new(kind))
        }
    };
    FieldSymbol {
        ident: node.ident.value.intern(pool),
        kind,
        is_padding: false,
    }
}

fn resolve_fields(
    context: &mut Context,
    header: &[FieldSymbol],
    nodes: &[FieldNode],
    max_align: Option<u8>,
    is_object: bool,
) -> Box<[FieldSymbol]> {
    let mut fields = Vec::with_capacity(nodes.len() + header.len());
    let calc = LayoutCalculator::new(
        max_align,
        header
            .iter()
            .copied()
            .chain(nodes.iter().map(|f| {
                resolve_field(
                    &mut context.diag,
                    &mut context.pool,
                    &mut context.table,
                    f,
                    is_object,
                )
            }))
            .collect::<Vec<_>>()
            .into_iter(),
    );

    for (item, _) in calc {
        let symbol = match item {
            LayoutItem::Field(s) => s,
            LayoutItem::Padding { index, size } => FieldSymbol {
                ident: context.pool.get_id_or_add(&format!("__padding{index}")),
                kind: FieldType::StaticArray(ArraySymbol::new(T_U8_ID, size)),
                is_padding: true,
            },
        };

        fields.push(symbol);
    }

    fields.into_boxed_slice()
}

fn resolve_struct(context: &mut Context, node: &StructNode) {
    let fields = resolve_fields(context, &[], &node.fields, None, false);
    let meta = get_meta(Some(32), &fields).or_recover(&mut context.diag);
    let s = StructSymbol {
        ident: node.ident.value.intern(&mut context.pool),
        fields,
    };
    context.table.register_type(meta, Type::Struct(s));
}

fn resolve_object(context: &mut Context, node: &ObjectNode) {
    let fields = resolve_fields(context, &[], &node.fields, None, true);
    let meta = get_meta(None, &fields).or_recover(&mut context.diag);
    let s = ObjectSymbol {
        ident: node.ident.value.intern(&mut context.pool),
        fields,
    };
    context.table.register_type(meta, Type::Object(s));
}

fn resolve_message_modififer(diag: &mut Diagnostics, modifier: &ModifierNode) -> Direction {
    match Direction::from_str(modifier.ident.value) {
        Ok(d) => d,
        Err(_) => todo!(),
    }
}

#[derive(Default)]
struct Metadata {
    from: Option<Option<Version>>,
    until: Option<Option<Version>>,
    reason: Option<String>,
}

#[derive(Default)]
struct Attributes {
    extension: Option<Metadata>,
    experimental: Option<Metadata>,
    builtin: bool,
}

impl Attributes {
    fn get_api_status(&self, context: &mut Context) -> ApiStatus {
        match (&self.experimental, &self.extension) {
            (Some(_), Some(_)) => {
                context
                    .diag
                    .report_err(SemanticError::ConflictingAttributes {
                        a: "Experimental",
                        b: "Extension",
                    });
                ApiStatus::Stable
            }
            (Some(_), None) => ApiStatus::Experimental,
            (None, Some(_)) => ApiStatus::Extension,
            (None, None) => ApiStatus::Stable,
        }
    }
}

fn consume_string<'src>(
    context: &mut Context,
    value: &AttributeParameterValueNode<'src>,
) -> Option<(&'src str, Span)> {
    match value {
        AttributeParameterValueNode::Number(n) => {
            context.diag.report_err(SyntaxError::UnexpectedToken {
                expected: "string",
                span: n.span.into(),
            });
            None
        }
        AttributeParameterValueNode::String(n) => Some((n.value, n.span)),
    }
}

fn consume_version(context: &mut Context, node: &NamedAttributeParameterNode) -> Option<Version> {
    if let Some((v, s)) = consume_string(context, &node.value) {
        Some(parse_version(&mut context.diag, v, s))
    } else {
        None
    }
}

fn consume_metadata(context: &mut Context, node: Option<&AttributeParameterListNode>) -> Metadata {
    let mut metadata = Metadata::default();
    if let Some(params) = &node {
        match params {
            AttributeParameterListNode::Unnamed(_) => {
                todo!("unsupported")
                //return None;
            }
            AttributeParameterListNode::Named(n) => {
                for n in n {
                    match n.ident.value {
                        "from" => {
                            if metadata.from.is_some() {
                                todo!("parameter already defined");
                                //continue;
                            }

                            // TODO: version can't be lower than protocol version
                            // TODO: version can't be more than until version
                            metadata.from = Some(consume_version(context, n));
                        }
                        "until" => {
                            if metadata.until.is_some() {
                                todo!("parameter already defined");
                                //continue;
                            }

                            metadata.until = Some(consume_version(context, n));
                        }
                        "reason" => {
                            if metadata.reason.is_some() {
                                todo!("parameter already defined");
                            }

                            metadata.reason =
                                if let Some((v, _)) = consume_string(context, &n.value) {
                                    Some(v.to_string())
                                } else {
                                    Some(String::new())
                                };
                        }
                        _ => todo!("unknown parameter"),
                    }
                }
                metadata
            }
        }
    } else {
        metadata
    }
}

fn resolve_attribute_list(context: &mut Context, list: &[AttributeListNode]) -> Attributes {
    let mut attributes = Attributes::default();

    for root in list.iter().flat_map(|n| &n.inner) {
        match root.ident.value {
            "Extension" => {
                if attributes.extension.is_some() {
                    context
                        .diag
                        .report_err(SemanticError::DuplicateAttributeError {
                            name: "Extension",
                            span: root.ident.span.into(),
                        });
                    continue;
                }

                attributes.extension = Some(consume_metadata(context, root.parameters.as_ref()));
            }
            "Experimental" => {
                if attributes.experimental.is_some() {
                    context
                        .diag
                        .report_err(SemanticError::DuplicateAttributeError {
                            name: "Experimental",
                            span: root.ident.span.into(),
                        });
                    continue;
                }

                attributes.experimental = Some(consume_metadata(context, root.parameters.as_ref()));
            }
            "Builtin" => {
                if attributes.builtin {
                    context
                        .diag
                        .report_err(SemanticError::DuplicateAttributeError {
                            name: "Builtin",
                            span: root.ident.span.into(),
                        });
                    continue;
                }

                attributes.builtin = true;
                if root.parameters.is_some() {
                    panic!("'Builtin' attribute doesn't accept any parameter")
                }
            }
            other => {
                context.diag.report_err(SemanticError::UnknownAttribute {
                    name: other.to_string(),
                    span: root.ident.span.into(),
                });
            }
        }
    }

    attributes
}

fn dst(context: &mut Context) -> FieldSymbol {
    FieldSymbol {
        ident: "dst".intern(&mut context.pool),
        kind: FieldType::Special(SpecialType::SubscriberId),
        is_padding: false,
    }
}

fn protocol(context: &mut Context) -> FieldSymbol {
    FieldSymbol {
        ident: "protocol".intern(&mut context.pool),
        kind: FieldType::Special(SpecialType::ProtocolId),
        is_padding: false,
    }
}

fn src(context: &mut Context) -> FieldSymbol {
    FieldSymbol {
        ident: "src".intern(&mut context.pool),
        kind: FieldType::Special(SpecialType::SubscriberId),
        is_padding: false,
    }
}

fn kind(context: &mut Context) -> FieldSymbol {
    FieldSymbol {
        ident: "kind".intern(&mut context.pool),
        kind: FieldType::Type(T_U8_ID),
        is_padding: false,
    }
}

fn message_header(context: &mut Context) -> [FieldSymbol; 4] {
    [dst(context), protocol(context), src(context), kind(context)]
}

fn stream_message_fields(context: &mut Context) -> Box<[FieldSymbol]> {
    Box::new([
        dst(context),
        protocol(context),
        src(context),
        kind(context),
        FieldSymbol {
            ident: "meta".intern(&mut context.pool),
            kind: FieldType::Type(T_U8_ID),
            is_padding: false,
        },
        FieldSymbol {
            ident: "data".intern(&mut context.pool),
            kind: FieldType::StaticArray(ArraySymbol::new(
                T_U8_ID,
                Size::try_new(MAX_PAYLOAD - 1).expect("Unreachable!"),
            )),
            is_padding: false,
        },
    ])
}

pub fn get_meta(max_size: Option<u8>, fields: &[FieldSymbol]) -> Result<TypeMeta, SemanticError> {
    let mut max_align = 1u64;
    let mut total_size = compute_layout(max_size, fields, |_, l| {
        if l.align > max_align {
            max_align = l.align;
        }
    });

    let tail_padding = (max_align - (total_size % max_align)) % max_align;
    total_size += tail_padding;

    TypeMeta::try_new(total_size, max_align)
}

const MESSAGE_SIZE: u64 = 32;

fn calculate_layout_range(fields: &[FieldSymbol]) -> SerializationLayout {
    let mut layout = SerializationLayout::default();
    let mut array_write = vec![];

    let mut to = 0;

    for field in fields {
        match field.kind() {
            FieldType::Type(s) => {
                to += s.size().value();
            }
            FieldType::StaticArray(s) => {
                to += s.total_size();
            }
            FieldType::HeapArray(s) => {
                if to != 0 {
                    layout.push(LayoutEntry::Slice { to, kind: s.kind() });
                }

                layout.push(LayoutEntry::ArrayLength {
                    name: field.ident(),
                });
                array_write.push(LayoutEntry::ArrayWrite {
                    name: field.ident(),
                });

                to = 0;
            }
            FieldType::Special(_) => unreachable!(),
        }
    }

    if to != 0 {
        layout.push(LayoutEntry::LastSlice { to });
    }

    for entry in array_write {
        layout.push(entry);
    }

    layout
}

fn resolve_message(context: &mut Context, node: &MessageNode) {
    let attributes = resolve_attribute_list(context, &node.attributes);
    let api = attributes.get_api_status(context);
    let symbol = match node.kind {
        MessageNodeType::Single => {
            let header = message_header(context);
            let fields = resolve_fields(context, &header, &node.fields, Some(32), false);
            let kind = context.issue_kind();
            let meta = get_meta(Some(MESSAGE_SIZE as u8), &fields).or_recover(&mut context.diag);

            if meta.size().value() > MESSAGE_SIZE {
                context.diag.report_err(SemanticError::InvalidSize {
                    actual: meta.size().value(),
                });
            }

            MessageSymbol {
                ident: node.ident.value.intern(&mut context.pool),
                fields,
                direction: resolve_message_modififer(&mut context.diag, &node.modifier),
                kind,
                api,
                message_type: MessageSymbolType::Single,
            }
        }
        MessageNodeType::Stream => {
            let basic_fields = stream_message_fields(context);
            let content_fields = resolve_fields(context, &[], &node.fields, None, true);
            let layout = calculate_layout_range(&content_fields);
            let kind = context.issue_kind();

            MessageSymbol {
                ident: node.ident.value.intern(&mut context.pool),
                fields: basic_fields,
                direction: resolve_message_modififer(&mut context.diag, &node.modifier),
                kind,
                api,
                message_type: MessageSymbolType::Stream {
                    serialization_layout: layout,
                    lightweight_content: content_fields,
                },
            }
        }
    };

    context.messages.push(symbol);
}

fn resolve_complex_types(tree: &SyntaxTree, context: &mut Context) {
    let (transform, graph) = build_dependency_graph(&mut context.pool, &context.table, tree);
    match toposort(&graph, None) {
        Ok(order) => {
            for ty in order
                .into_iter()
                .rev()
                .map(|i| transform.get(&i).unwrap())
                .map(|i| &tree.complex[*i])
            {
                match ty {
                    ComplexType::Struct(n) => resolve_struct(context, n),
                    ComplexType::Object(n) => resolve_object(context, n),
                    ComplexType::Message(n) => resolve_message(context, n),
                }
            }
        }
        Err(_) => todo!(),
    }
}

fn parse_version(diag: &mut Diagnostics, value: &str, span: Span) -> Version {
    Version::from_str(value)
        .map_err(|_| SyntaxError::InvalidVersionFormat { span: span.into() })
        .or_recover(diag)
}

struct Directives {
    name: String,
    version: Version,
    access: Access,
    schema: u32,
}

fn resolve_directive_list(context: &mut Context, nodes: &[DirectiveNode]) -> Directives {
    let mut name = None;
    let mut version = None;
    let mut access = None;
    let mut schema = None;

    for root in nodes {
        match root.ident.value {
            "name" if name.is_none() => match &root.value {
                DefineValueNode::String(n) => {
                    name = Some(n.value.to_string());
                }
                DefineValueNode::Number(_) => {
                    todo!("unexpected expression")
                }
            },
            "name" if name.is_some() => {
                todo!("attribute defined multiple times");
            }
            "version" if version.is_none() => match &root.value {
                DefineValueNode::String(n) => {
                    version = Some(parse_version(&mut context.diag, n.value, n.span));
                }
                DefineValueNode::Number(_) => {
                    todo!("unexpected expression")
                }
            },
            "version" if version.is_some() => {
                todo!("attribute defined multiple times");
            }
            "access" if access.is_none() => match &root.value {
                DefineValueNode::String(n) => {
                    access = Some(Access::from_str(n.value).unwrap());
                }
                DefineValueNode::Number(_) => {
                    todo!("unexpected expression")
                }
            },
            "access" if access.is_some() => {
                todo!("attribute defined multiple times");
            }
            "schema" if schema.is_none() => match &root.value {
                DefineValueNode::String(_) => {
                    todo!("unexpected expression")
                }
                DefineValueNode::Number(n) => {
                    schema = Some(n.value.parse::<u32>().unwrap());
                }
            },
            "schema" if schema.is_some() => {
                todo!("attribute defined multiple times");
            }
            _ => todo!("unknown define node"),
        }
    }

    Directives {
        name: name.unwrap(),
        version: version.unwrap(),
        access: access.unwrap(),
        schema: schema.unwrap(),
    }
}

pub struct Api {
    stable: [u8; 32],
    extension: Box<[u8; 32]>,
    experimental: Box<[u8; 32]>,
}

#[derive(Debug)]
pub struct SemanticModel {
    name: String,
    version: Version,
    access: Access,
    schema: u32,
    table: FinishedSymbolTable,
    messages: Vec<MessageSymbol>,
    pool: StringPool,
}

impl SemanticModel {
    pub fn compile(source: &str) -> Result<Self, Diagnostics> {
        let mut diag = Diagnostics::default();
        let tree: SyntaxTree = SyntaxTreeBuilder::new(source).run(&mut diag);
        if diag.has_errors() {
            return Err(diag);
        }

        let mut context = Context::new(tree.type_count() as u16);
        let directives = resolve_directive_list(&mut context, &tree.directives);
        resolve_flags(&mut context, &tree.flags);
        resolve_enums(&mut context, &tree.enums);
        resolve_bitsets(&mut context, &tree.bitsets);
        resolve_complex_types(&tree, &mut context);

        if context.diag.has_errors() {
            return Err(context.diag);
        }

        Ok(Self {
            name: directives.name,
            version: directives.version,
            access: directives.access,
            schema: directives.schema,
            table: context.table.finish(),
            messages: context.messages,
            pool: context.pool,
        })
    }

    #[must_use]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    #[must_use]
    pub const fn access(&self) -> Access {
        self.access
    }

    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }

    #[must_use]
    pub const fn messages(&self) -> &[MessageSymbol] {
        self.messages.as_slice()
    }

    #[must_use]
    pub const fn pool(&self) -> &StringPool {
        &self.pool
    }

    #[must_use]
    pub fn types(&self) -> &[Type] {
        self.table.types()
    }

    #[must_use]
    pub const fn table(&self) -> &FinishedSymbolTable {
        &self.table
    }
}

#[cfg(test)]
mod tests {
    const SAMPLE: &str = include_str!("../../../system/system.cmp");
    const SAMPLE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../system/system.cmp");

    #[test]
    fn test() {
        match super::SemanticModel::compile(SAMPLE) {
            Ok(model) => {
                dbg!(model);
            }
            Err(d) => {
                d.print(SAMPLE_PATH, SAMPLE);
                panic!();
            }
        }
    }
}
