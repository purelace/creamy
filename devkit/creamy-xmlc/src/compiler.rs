use std::{cell::RefCell, num::NonZeroU8};

use as_guard::AsGuard;
use creamy_utils::strpool::StringPool;
use strum::EnumCount;

use crate::{
    ProtocolDefinition, StringPoolResolver,
    constraints::MAX_PAYLOAD,
    diagnostics::Diagnostics,
    error::{ProtocolErrorExt, SemanticError},
    model::{
        definition::Direction,
        storage::SymbolStorage,
        symbols::{
            ArraySymbol, BitsetSymbol, BitsetValueSymbol, EnumSymbol, FieldSymbol, FieldType,
            FlagsSymbol, GlobalTypesSymbol, GroupSymbol, MessageSymbol, MessageSymbolType,
            NumericSymbol, OptionSymbol, PrimitiveRepr, StreamPayloadFieldSymbol, StreamSymbol,
            StructSymbol, Type, VariantSymbol,
        },
    },
    table::{TypeMeta, TypeTable},
    tokenizer::tokenize,
    tree::{
        ProtocolTree,
        nodes::{
            BitsetNode, BitsetValueNode, EnumNode, FieldNode, FieldTypeNode, FlagsNode,
            GlobalTypesNode, GroupNode, MessageNode, MessageNodeType, OptionNode, StreamNode,
            StreamPayloadFieldNode, StructNode, VariantNode,
        },
    },
    utils::{BitsetValuesRange, FieldsRange, Range, Size, TypesRange},
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
    storage: SymbolStorage,
    diag: &'a mut Diagnostics,
    pool: &'a StringPool,
}

impl<'a> DefinitionBuilder<'a> {
    fn new(diag: &'a mut Diagnostics, pool: &'a StringPool) -> Self {
        Self {
            storage: SymbolStorage::default(),
            diag,
            pool,
        }
    }

    fn resolve_global_types(
        &mut self,
        tree: &ProtocolTree,
        tt: &mut TypeTable,
        group: GlobalTypesNode,
    ) -> GlobalTypesSymbol {
        let mut resolver = Resolver::new(
            self.diag,
            tt,
            &mut self.storage,
            self.pool,
            //FIX: we don't need a group for the global types
            NonZeroU8::new(1).unwrap(),
        );

        resolver.resolve_flags(
            tree.storage.get_node_range(group.flags()),
            tree.storage.get_node_slice::<OptionNode>(),
        );

        resolver.resolve_bitsets(
            tree.storage.get_node_range(group.bitsets()),
            tree.storage.get_node_slice::<BitsetValueNode>(),
        );

        resolver.resolve_enums(
            tree.storage.get_node_range(group.enums()),
            tree.storage.get_node_slice::<VariantNode>(),
        );

        resolver.resolve_structs(
            tree.storage.get_node_range(group.structs()),
            tree.storage.get_node_slice::<FieldNode>(),
        );

        let start = group.flags().start() + NumericSymbol::COUNT as u16;

        let total_len = group.flags().len()
            + group.bitsets().len()
            + group.enums().len()
            + group.structs().len();

        GlobalTypesSymbol::new(TypesRange::new(start, total_len))
    }

    fn resolve_group(
        &mut self,
        tree: &ProtocolTree,
        tt: &mut TypeTable,
        group: GroupNode,
        group_id: NonZeroU8,
    ) -> GroupSymbol {
        let mut resolver = Resolver::new(self.diag, tt, &mut self.storage, self.pool, group_id);
        resolver.resolve_flags(
            tree.storage.get_node_range(group.flags()),
            tree.storage.get_node_slice::<OptionNode>(),
        );

        resolver.resolve_bitsets(
            tree.storage.get_node_range(group.bitsets()),
            tree.storage.get_node_slice::<BitsetValueNode>(),
        );

        resolver.resolve_enums(
            tree.storage.get_node_range(group.enums()),
            tree.storage.get_node_slice::<VariantNode>(),
        );

        resolver.resolve_structs(
            tree.storage.get_node_range(group.structs()),
            tree.storage.get_node_slice::<FieldNode>(),
        );

        resolver.resolve_messages(
            tree.storage.get_node_range(group.messages()),
            tree.storage.get_node_slice::<FieldNode>(),
            tree.storage.get_node_slice::<StreamPayloadFieldNode>(),
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

fn run(diag: &mut Diagnostics, pool: &StringPool, tree: ProtocolTree) -> ProtocolDefinition {
    let mut tt = TypeTable::new(tree.storage.len_of::<GroupNode>() as u8, tree.type_count());
    let mut builder = DefinitionBuilder::new(diag, pool);

    let global = builder.resolve_global_types(&tree, &mut tt, tree.global);

    for (idx, group) in tree
        .storage
        .get_node_slice::<GroupNode>()
        .iter()
        .enumerate()
    {
        let group_id = NonZeroU8::new(idx.safe_as::<u8>() + 1).expect("Group limit exceeded");
        let symbol = builder.resolve_group(&tree, &mut tt, *group, group_id);
        assert!(builder.storage.add_symbol(symbol), "Unreachable!");
    }

    ProtocolDefinition::new(
        tree.name,
        tree.version,
        global,
        builder.storage,
        tt.finish(),
    )
}

struct Resolver<'a> {
    diag: &'a mut Diagnostics,
    tt: &'a mut TypeTable,
    storage: &'a mut SymbolStorage,
    pool: &'a StringPool,
    group: NonZeroU8,
}

impl<'a> Resolver<'a> {
    const fn new(
        diag: &'a mut Diagnostics,
        tt: &'a mut TypeTable,
        storage: &'a mut SymbolStorage,
        pool: &'a StringPool,
        group: NonZeroU8,
    ) -> Self {
        Self {
            diag,
            tt,
            storage,
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

    fn resolve_fields(&mut self, from: &[FieldNode]) {
        for field in from {
            let symbol = self.resolve_field(*field);
            assert!(self.storage.add_symbol(symbol));
        }
    }

    fn resolve_enums(&mut self, from: &[EnumNode], variants: &[VariantNode]) {
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
                        self.storage
                            .add_symbol(VariantSymbol::new(variant.name(), value)),
                        "Unreachable!"
                    );
                }
            }

            let meta = symbol.meta().or_recover(self.diag);
            self.tt.register_type(self.group, meta, Type::Enum(symbol));
        }
    }

    fn resolve_structs(&mut self, structs: &[StructNode], from: &[FieldNode]) {
        let mut to_resolve = structs.len();
        // Устанавливаем любое значение отличное от нуля чтобы пройти первую итерацию цикла
        let mut last_resolved = 1;
        while to_resolve != 0 {
            for s in structs {
                if self.tt.contains_name(s.name()) {
                    continue;
                }

                let from = &from[s.fields().as_range()];
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
                    self.resolve_fields(from);
                    let len = self.storage.get_symbol_slice::<FieldSymbol>().len() as u16;
                    let meta = ProtocolDefinition::get_struct_meta(self.storage.get_symbol_range(
                        FieldsRange::new(len - u16::from(s.fields().len()), s.fields().len()),
                    ))
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
        messages: &[MessageNodeType],
        fields: &[FieldNode],
    ) -> MessageSymbol {
        let from = &fields[m.fields().as_range()];

        let mut has_errors = false;
        if let Some(failed) = from
            .iter()
            .find(|f| !self.tt.contains_name(f.kind().type_name()))
        {
            let name = m.name().resolve(self.pool);
            let kind = failed.kind().type_name();
            let error = if m.name() == kind {
                SemanticError::SelfReference(name.to_string())
            } else if messages.iter().any(|m| m.name() == kind) {
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
            MessageSymbol::new(
                m.name(),
                FieldsRange::new(0, 0),
                Direction::Incoming,
                m.kind(),
            )
        } else {
            self.resolve_fields(from);

            let len = self.storage.len_of::<FieldSymbol>() as u16;

            // Check size and align
            ProtocolDefinition::get_message_meta(self.storage.get_symbol_range(FieldsRange::new(
                len - u16::from(m.fields().len()),
                m.fields().len(),
            )))
            .or_recover(self.diag);

            MessageSymbol::new(m.name(), m.fields(), m.direction(), m.kind())
        }
    }

    fn resolve_payload_fields(&mut self, from: &[StreamPayloadFieldNode]) {
        for field in from {
            match field {
                StreamPayloadFieldNode::Field(field) => {
                    let symbol = self.resolve_field(*field);
                    assert!(
                        self.storage
                            .add_symbol(StreamPayloadFieldSymbol::Field(symbol))
                    );
                } //StreamPayloadFieldNode::Array(array) => {
                  //    let ty = self.tt.get_type_by_name(array.kind()).unwrap();
                  //    assert!(
                  //        to.push(StreamPayloadFieldSymbol::Array(ArrayFieldSymbol::new(
                  //            array.name(),
                  //            ty,
                  //            array.len()
                  //        ))),
                  //        "Unreachable!"
                  //    );
                  //}
            }
        }
    }

    fn resolve_stream_message(
        &mut self,
        m: StreamNode,
        m_from: &[MessageNodeType],
        f_from: &[FieldNode],
        s_from: &[StreamPayloadFieldNode],
    ) -> StreamSymbol {
        if let Some(start) = m.head() {
            let from = &f_from[start.as_range()];
            self.resolve_fields(from);
            //TODO
        }

        // check payload
        {
            let from = &s_from[m.payload().as_range()];
            self.resolve_payload_fields(from);
        }

        // check tail
        if let Some(end) = m.tail() {
            let from = &f_from[end.as_range()];
            self.resolve_fields(from);
            //TODO
        }

        //let len = f_to.len() as u16;

        // Check size and align
        //ProtocolDefinition::get_message_meta(
        //    &f_to[FieldsRange::new(len - u16::from(m.payload().len()), m.payload().len())],
        //)
        //.or_recover(self.diag);

        StreamSymbol::new(
            m.name(),
            m.direction(),
            m.timeout(),
            m.kind(),
            m.head(),
            //payload_range,
            m.payload(),
            m.tail(),
        )
    }

    fn resolve_messages(
        &mut self,
        m_from: &[MessageNodeType],
        f_from: &[FieldNode],
        s_from: &[StreamPayloadFieldNode],
    ) {
        for m in m_from {
            let kind = match m {
                MessageNodeType::Single(m) => {
                    MessageSymbolType::Single(self.resolve_single_message(*m, m_from, f_from))
                }
                MessageNodeType::Stream(m) => MessageSymbolType::Stream(
                    self.resolve_stream_message(*m, m_from, f_from, s_from),
                ),
            };
            assert!(self.storage.add_symbol(kind), "Unreachable!");
        }
    }

    fn resolve_flags(&mut self, flags: &[FlagsNode], from: &[OptionNode]) {
        for f in flags {
            let slice = &from[f.options().as_range()];

            for option in slice {
                assert!(
                    self.storage.add_symbol(OptionSymbol::new(option.ident())),
                    "Unreachable!"
                );
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

    fn resolve_bitsets(&mut self, from: &[BitsetNode], values: &[BitsetValueNode]) {
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
                    self.storage.add_symbol(BitsetValueSymbol::new(
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
                BitsetSymbol::new(f.ident(), BitsetValuesRange::new(0, 0))
            } else {
                BitsetSymbol::new(f.ident(), f.values())
            };

            self.tt
                .register_type(self.group, meta, Type::Bitset(symbol));
        }
    }
}
