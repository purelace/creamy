use std::{cell::RefCell, num::NonZeroU8};

use as_guard::AsGuard;
use creamy_utils::strpool::StringPool;
use strum::EnumCount;

use crate::{
    ProtocolDefinition, StringPoolResolver,
    constraints::MAX_PAYLOAD,
    diagnostics::Diagnostics,
    error::{ProtocolErrorExt, SemanticError},
    model::symbols::{
        ArrayFieldSymbol, ArraySymbol, BitsetSymbol, BitsetValueSymbol, EnumSymbol, FieldSymbol,
        FieldType, FlagsSymbol, GroupSymbol, MessageSymbol, MessageSymbolType, NumericSymbol,
        OptionSymbol, PrimitiveRepr, StreamPayloadFieldSymbol, StreamSymbol, StructSymbol, Type,
        VariantSymbol,
    },
    nodes::{
        BValueNode, BitsetNode, EnumNode, FieldNode, FieldTypeNode, FlagsNode, GroupNode,
        MessageNode, MessageNodeType, OptionNode, StreamNode, StreamPayloadFieldNode, StructNode,
        VariantNode,
    },
    table::{TypeMeta, TypeTable},
    tokenizer::tokenize,
    tree::ProtocolTree,
    utils::{BValuesRange, BoundedVec, FieldsRange, Range, Size, TypesRange},
};

pub fn compile(pool: &mut StringPool, content: &str) -> Result<ProtocolDefinition, Diagnostics> {
    let content = content.trim();
    let diag = RefCell::new(Diagnostics::default());
    let tokens = tokenize(content, &diag);
    let mut diag = diag.into_inner();
    if diag.has_errors() {
        return Err(diag);
    }

    let tree = ProtocolTree::new(tokens, pool, &mut diag);
    if diag.has_errors() {
        return Err(diag);
    }

    let def = run(&mut diag, pool, tree);
    if diag.has_errors() {
        Err(diag)
    } else {
        Ok(def)
    }
}

pub struct DefinitionBuilder<'a> {
    groups: BoundedVec<GroupSymbol>,
    messages: BoundedVec<MessageSymbolType>,
    values: BoundedVec<BitsetValueSymbol>,
    options: BoundedVec<OptionSymbol>,
    variants: BoundedVec<VariantSymbol>,
    fields: BoundedVec<FieldSymbol>,

    // TODO:
    // Тут надо пересчитать максимальное количество типов.
    payload: BoundedVec<StreamPayloadFieldSymbol>,

    diag: &'a mut Diagnostics,
    pool: &'a StringPool,
}

impl<'a> DefinitionBuilder<'a> {
    fn new(diag: &'a mut Diagnostics, pool: &'a StringPool, tree: &ProtocolTree) -> Self {
        Self {
            values: BoundedVec::with_capacity(tree.bvalues.len()),
            options: BoundedVec::with_capacity(tree.options.len()),
            variants: BoundedVec::with_capacity(tree.variants.len()),
            groups: BoundedVec::with_capacity(tree.groups.len()),
            fields: BoundedVec::with_capacity(tree.fields.len()),
            messages: BoundedVec::with_capacity(tree.messages.len()),
            payload: BoundedVec::with_capacity(tree.payload.len()),
            diag,
            pool,
        }
    }

    fn resolve_group(
        &mut self,
        tree: &ProtocolTree,
        tt: &mut TypeTable,
        group: GroupNode,
        group_id: NonZeroU8,
    ) -> GroupSymbol {
        let mut resolver = Resolver::new(self.diag, tt, self.pool, group_id);
        resolver.resolve_flags(&tree.flags[group.flags()], &tree.options, &mut self.options);

        resolver.resolve_bitsets(
            &tree.bitsets[group.bitsets()],
            &tree.bvalues,
            &mut self.values,
        );

        resolver.resolve_enums(
            &tree.enums[group.enums()],
            &tree.variants,
            &mut self.variants,
        );

        resolver.resolve_structs(
            &tree.structs[group.structs()],
            &tree.fields,
            &mut self.fields,
        );

        resolver.resolve_messages(
            &tree.messages[group.messages()],
            &mut self.messages,
            &tree.fields,
            &mut self.fields,
            &tree.payload,
            &mut self.payload,
        );

        let start = group.flags().start() + NumericSymbol::COUNT as u16;

        let total_len = group.flags().len()
            + group.bitsets().len()
            + group.enums().len()
            + group.structs().len();

        GroupSymbol::new(
            group.name(),
            group.access(),
            group.messages(),
            TypesRange::new(start, total_len),
        )
    }
}

fn run(diag: &mut Diagnostics, pool: &StringPool, mut tree: ProtocolTree) -> ProtocolDefinition {
    let mut tt = TypeTable::new(tree.groups.len() as u8, tree.type_count());
    let mut builder = DefinitionBuilder::new(diag, pool, &tree);

    let global = builder.resolve_group(&tree, &mut tt, tree.global, NonZeroU8::new(1).unwrap());

    let mut groups = std::mem::take(&mut tree.groups);
    for (idx, group) in groups.drain(..).enumerate() {
        let group_id = NonZeroU8::new(idx.safe_as::<u8>() + 1).expect("Group limit exceeded");
        let symbol = builder.resolve_group(&tree, &mut tt, group, group_id);
        assert!(builder.groups.push(symbol), "Unreachable!");
    }

    ProtocolDefinition::new(
        tree.name,
        tree.version,
        global,
        builder.groups,
        builder.messages,
        builder.values,
        builder.options,
        builder.variants,
        builder.fields,
        builder.payload,
        tt.finish(),
    )
}

struct Resolver<'a> {
    diag: &'a mut Diagnostics,
    tt: &'a mut TypeTable,
    pool: &'a StringPool,
    group: NonZeroU8,
}

impl<'a> Resolver<'a> {
    const fn new(
        diag: &'a mut Diagnostics,
        tt: &'a mut TypeTable,
        pool: &'a StringPool,
        group: NonZeroU8,
    ) -> Self {
        Self {
            diag,
            tt,
            pool,
            group,
        }
    }

    fn resolve_field(&mut self, node: FieldNode) -> FieldSymbol {
        let kind = match node.kind() {
            FieldTypeNode::Type(name) => FieldType::Type(self.tt.get_type_by_name(name).unwrap()),
            FieldTypeNode::Array(array) => {
                let kind = self.tt.get_type_by_name(array.kind()).unwrap();
                let size = Size::new(array.size());
                FieldType::Array(ArraySymbol::new(kind, size.or_recover(self.diag)))
            }
        };
        FieldSymbol::new(node.name(), kind)
    }

    fn resolve_fields(&mut self, from: &[FieldNode], to: &mut BoundedVec<FieldSymbol>) {
        for field in from {
            let symbol = self.resolve_field(*field);
            assert!(to.push(symbol));
        }
    }

    fn resolve_enums(
        &mut self,
        from: &[EnumNode],
        variants: &[VariantNode],
        to: &mut BoundedVec<VariantSymbol>,
    ) {
        for e in from {
            let result = PrimitiveRepr::try_from(e.repr().resolve(self.pool));
            let mut error = false;
            let repr = match result {
                Ok(repr) => repr,
                Err(err) => {
                    error = true;
                    self.diag.report_err(err);
                    PrimitiveRepr::U8
                }
            };

            let symbol = EnumSymbol::new(e.name(), repr, e.variants());

            if e.variants().len() == 0 {
                self.diag.report_err(SemanticError::ZeroSizedType);
            }

            //TODO: check duplicates

            if !error {
                for variant in &variants[e.variants().as_range()] {
                    let value = variant.value();
                    if !repr.is_valid_value(value) {
                        self.diag
                            .report_err(SemanticError::EnumVariantValueOutOfRange {
                                value,
                                min: repr.get_min(),
                                max: repr.get_max(),
                            });
                        continue;
                    }
                    assert!(
                        to.push(VariantSymbol::new(variant.name(), value)),
                        "Unreachable!"
                    );
                }
            }

            let meta = symbol.meta().or_recover(self.diag);
            self.tt.register_type(self.group, meta, Type::Enum(symbol));
        }
    }

    fn resolve_structs(
        &mut self,
        structs: &[StructNode],
        f_from: &[FieldNode],
        f_to: &mut BoundedVec<FieldSymbol>,
    ) {
        let mut to_resolve = structs.len();
        // Устанавливаем любое значение отличное от нуля чтобы пройти первую итерацию цикла
        let mut last_resolved = 1;
        while to_resolve != 0 {
            for s in structs {
                if self.tt.contains_name(s.name()) {
                    continue;
                }

                let from = &f_from[s.fields().as_range()];
                let mut has_errors = false;

                if let Some(failed) = from
                    .iter()
                    .find(|f| !self.tt.contains_name(f.kind().type_name()))
                {
                    if last_resolved == 0 {
                        let name = s.name().resolve(self.pool);
                        let kind = failed.kind().type_name();
                        let error = if s.name() == kind {
                            SemanticError::SelfReference(name.to_string())
                        //} else if messages.iter().any(|m| m.name() == kind) {
                        //    ProtocolError::MessageReference(name.to_string())
                        } else {
                            SemanticError::CannotResolveTypeFieldNotFound {
                                from: name.to_string(),
                                kind: kind.resolve(self.pool).to_string(),
                            }
                        };
                        self.diag.report_err(error);
                        has_errors = true;
                    } else {
                        last_resolved = 0;
                        continue;
                    }
                }

                if has_errors {
                    self.tt.register_type(
                        self.group,
                        TypeMeta::S1B_A1B,
                        Type::Struct(StructSymbol::new(s.name(), FieldsRange::new(0, 0))),
                    );
                } else {
                    self.resolve_fields(from, f_to);
                    let len = f_to.len() as u16;
                    let meta = ProtocolDefinition::get_struct_meta(
                        &f_to
                            [FieldsRange::new(len - u16::from(s.fields().len()), s.fields().len())],
                    )
                    .or_recover(self.diag);
                    let s = StructSymbol::new(s.name(), s.fields());

                    self.tt.register_type(self.group, meta, Type::Struct(s));
                }

                to_resolve -= 1;
                last_resolved = 0;
            }
        }
    }

    fn resolve_single_message(
        &mut self,
        m: MessageNode,
        m_from: &[MessageNodeType],
        from: &[FieldNode],
        to: &mut BoundedVec<FieldSymbol>,
    ) -> MessageSymbol {
        let from = &from[m.fields().as_range()];

        let mut has_errors = false;
        if let Some(failed) = from
            .iter()
            .find(|f| !self.tt.contains_name(f.kind().type_name()))
        {
            let name = m.name().resolve(self.pool);
            let kind = failed.kind().type_name();
            let error = if m.name() == kind {
                SemanticError::SelfReference(name.to_string())
            } else if m_from.iter().any(|m| m.name() == kind) {
                SemanticError::MessageReference(name.to_string())
            } else {
                SemanticError::CannotResolveTypeFieldNotFound {
                    from: name.to_string(),
                    kind: kind.resolve(self.pool).to_string(),
                }
            };
            self.diag.report_err(error);
            has_errors = true;
        }

        if has_errors {
            MessageSymbol::new(m.name(), FieldsRange::new(0, 0), m.kind())
        } else {
            self.resolve_fields(from, to);

            let len = to.len() as u16;

            // Check size and align
            ProtocolDefinition::get_message_meta(
                &to[FieldsRange::new(len - u16::from(m.fields().len()), m.fields().len())],
            )
            .or_recover(self.diag);

            MessageSymbol::new(m.name(), m.fields(), m.kind())
        }
    }

    fn resolve_payload_fields(
        &mut self,
        from: &[StreamPayloadFieldNode],
        to: &mut BoundedVec<StreamPayloadFieldSymbol>,
    ) -> FieldsRange {
        let start = to.len();
        for field in from {
            match field {
                StreamPayloadFieldNode::Field(field) => {
                    let symbol = self.resolve_field(*field);
                    assert!(to.push(StreamPayloadFieldSymbol::Field(symbol)));
                }
                StreamPayloadFieldNode::Array(array) => {
                    let ty = self.tt.get_type_by_name(array.kind()).unwrap();
                    assert!(
                        to.push(StreamPayloadFieldSymbol::Array(ArrayFieldSymbol::new(
                            array.name(),
                            ty,
                            array.len()
                        ))),
                        "Unreachable!"
                    );
                }
            }
        }

        FieldsRange::new(start as u16, (to.len() - start) as u8)
    }

    fn resolve_stream_message(
        &mut self,
        m: StreamNode,
        m_from: &[MessageNodeType],
        f_from: &[FieldNode],
        f_to: &mut BoundedVec<FieldSymbol>,
        s_from: &[StreamPayloadFieldNode],
        s_to: &mut BoundedVec<StreamPayloadFieldSymbol>,
    ) -> StreamSymbol {
        if let Some(start) = m.start() {
            let from = &f_from[start.as_range()];
            self.resolve_fields(from, f_to);
            //TODO
        }

        //check payload
        let from = &s_from[m.payload().as_range()];
        let payload_range = self.resolve_payload_fields(from, s_to);

        //let len = f_to.len() as u16;

        // Check size and align
        //ProtocolDefinition::get_message_meta(
        //    &f_to[FieldsRange::new(len - u16::from(m.payload().len()), m.payload().len())],
        //)
        //.or_recover(self.diag);

        StreamSymbol::new(
            m.name(),
            m.timeout(),
            m.kind(),
            m.start(),
            //payload_range,
            m.payload(),
            m.end(),
        )
    }

    fn resolve_messages(
        &mut self,
        m_from: &[MessageNodeType],
        m_to: &mut BoundedVec<MessageSymbolType>,
        f_from: &[FieldNode],
        f_to: &mut BoundedVec<FieldSymbol>,
        s_from: &[StreamPayloadFieldNode],
        s_to: &mut BoundedVec<StreamPayloadFieldSymbol>,
    ) {
        for m in m_from {
            let kind = match m {
                MessageNodeType::Single(m) => {
                    MessageSymbolType::Single(self.resolve_single_message(*m, m_from, f_from, f_to))
                }
                MessageNodeType::Stream(m) => MessageSymbolType::Stream(
                    self.resolve_stream_message(*m, m_from, f_from, f_to, s_from, s_to),
                ),
            };
            assert!(m_to.push(kind), "Unreachable!");
        }
    }

    fn resolve_flags(
        &mut self,
        flags: &[FlagsNode],
        from: &[OptionNode],
        to: &mut BoundedVec<OptionSymbol>,
    ) {
        for f in flags {
            let slice = &from[f.options().as_range()];

            for option in slice {
                assert!(to.push(OptionSymbol::new(option.ident())), "Unreachable!");
            }

            let bits = slice.len();
            let bytes = bits.div_ceil(8);
            let meta = match bytes {
                1 => TypeMeta::S1B_A1B,
                2 => TypeMeta::S2B_A2B,
                3 | 4 => TypeMeta::S4B_A4B,
                5..=8 => TypeMeta::S8B_A8B,
                9..=16 => TypeMeta::S16B_A16B,
                _ => unreachable!(),
            };

            let symbol = FlagsSymbol::new(f.ident(), f.options());
            self.tt.register_type(self.group, meta, Type::Flags(symbol));
        }
    }

    fn resolve_bitsets(
        &mut self,
        from: &[BitsetNode],
        values: &[BValueNode],
        to: &mut BoundedVec<BitsetValueSymbol>,
    ) {
        const MAX_BITS: usize = MAX_PAYLOAD * 8;
        for f in from {
            let slice = &values[f.values().as_range()];

            let mut has_errors = false;
            let mut total_bits = 0;
            for value in slice {
                total_bits += value.bits();
                if value.bits() > MAX_BITS {
                    has_errors = true;
                    break;
                }
                if total_bits > MAX_BITS {
                    has_errors = true;
                    break;
                }

                let repr = self.tt.get_type_by_name(value.repr()).unwrap();

                assert!(
                    to.push(BitsetValueSymbol::new(
                        value.ident(),
                        repr,
                        value.bits() as u8
                    )),
                    "Unreachable!"
                );
            }

            let bytes = total_bits.div_ceil(8);
            let meta = TypeMeta::new(bytes as u8, 1).or_recover(self.diag);

            let symbol = if has_errors {
                BitsetSymbol::new(f.ident(), BValuesRange::new(0, 0))
            } else {
                BitsetSymbol::new(f.ident(), f.values())
            };

            self.tt
                .register_type(self.group, meta, Type::Bitset(symbol));
        }
    }
}
