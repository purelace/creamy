use std::{cell::RefCell, num::NonZeroU8};

use as_guard::AsGuard;
use creamy_protocol_model::{
    BitsetValuesRange, Direction, FieldsRange, Range, Size, TypeMeta, TypeTable, TypesRange,
    constraints::MAX_PAYLOAD,
    definition::ProtocolModel,
    error::SemanticError,
    storage::SymbolStorage,
    symbols::{
        ArraySymbol, BitsetSymbol, BitsetValueSymbol, EnumSymbol, F32_ID, F64_ID, FieldSymbol,
        FieldType, FlagsSymbol, GlobalTypesSymbol, GroupSymbol, HeapArraySymbol, I8_ID, I16_ID,
        I32_ID, I64_ID, I128_ID, MessageSymbol, MessageSymbolType,
        NumericSymbol::{self},
        OptionSymbol, PrimitiveRepr, StreamFieldSymbol, StreamFieldType, StreamSymbol,
        StructSymbol, Type, U8_ID, U16_ID, U32_ID, U64_ID, U128_ID, VariantSymbol,
    },
};
use creamy_utils::strpool::{StringPool, StringPoolResolver};
use strum::EnumCount;

use crate::{
    diagnostics::Diagnostics,
    error::ProtocolErrorExt,
    tokenizer::tokenize,
    tree::{
        ProtocolTree, StreamFieldNode, StreamFieldTypeNode,
        nodes::{
            BitsetNode, BitsetValueNode, EnumNode, FieldNode, FieldTypeNode, FlagsNode,
            GlobalTypesNode, GroupNode, MessageNode, MessageNodeType, OptionNode, StreamNode,
            StructNode, VariantNode,
        },
    },
};

fn insert_default_keywords(pool: &mut StringPool) {
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
}

pub fn compile(pool: &mut StringPool, content: &str) -> Result<ProtocolModel, Diagnostics> {
    insert_default_keywords(pool);

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
            tree.storage.get_node_slice::<StreamFieldNode>(),
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

fn run(diag: &mut Diagnostics, pool: &StringPool, tree: ProtocolTree) -> ProtocolModel {
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

    ProtocolModel::new(
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
                let size = Size::try_new(array.size());
                FieldType::Array(ArraySymbol::new(kind, size.or_recover(self.diag)))
            }
        };
        FieldSymbol::new(node.name(), kind)
    }

    /// Returns a range of resolved fields
    fn resolve_fields(&mut self, fields: &[FieldNode], range: FieldsRange) -> FieldsRange {
        let slice = &fields[range.as_range()];
        for field in slice {
            let symbol = self.resolve_field(*field);
            assert!(self.storage.add_symbol(symbol));
        }

        let len = self.storage.get_symbol_slice::<FieldSymbol>().len() as u16;
        FieldsRange::new(len - u16::from(range.len()), range.len())
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

    fn resolve_structs(&mut self, structs: &[StructNode], fields: &[FieldNode]) {
        let mut to_resolve = structs.len();
        // Устанавливаем true чтобы пройти первую итерацию цикла
        let mut is_any_resolved = true;
        while to_resolve != 0 {
            for s in structs {
                if self.tt.contains_name(s.name()) {
                    continue;
                }

                let mut has_errors = false;

                if let Some(failed) = fields[s.fields().as_range()]
                    .iter()
                    .find(|f| !self.tt.contains_name(f.kind().type_name()))
                {
                    if is_any_resolved {
                        is_any_resolved = false;
                        continue;
                    }

                    let name = s.name().resolve(self.pool);
                    let kind = failed.kind().type_name();
                    let error = if s.name() == kind {
                        SemanticError::SelfReference(name.to_string())
                    //} else if messages.iter().any(|m| m.name() == kind) {
                    //    ProtocolError::MessageReference(name.to_string())
                    } else {
                        dbg!(self.pool);
                        SemanticError::CannotResolveTypeFieldNotFound {
                            from: name.to_string(),
                            kind: kind.resolve(self.pool).to_string(),
                        }
                    };
                    self.diag.report_err(error);
                    has_errors = true;
                }

                if has_errors {
                    self.tt.register_type(
                        self.group,
                        TypeMeta::S1B_A1B,
                        Type::Struct(StructSymbol::new(s.name(), FieldsRange::new(0, 0))),
                    );
                } else {
                    let new_range = self.resolve_fields(fields, s.fields());
                    let meta =
                        ProtocolModel::get_struct_meta(self.storage.get_symbol_range(new_range))
                            .or_recover(self.diag);
                    let s = StructSymbol::new(s.name(), s.fields());

                    self.tt.register_type(self.group, meta, Type::Struct(s));
                }

                to_resolve -= 1;
                is_any_resolved = false;
            }
        }
    }

    fn resolve_single_message(
        &mut self,
        m: MessageNode,
        messages: &[MessageNodeType],
        fields: &[FieldNode],
    ) -> MessageSymbol {
        let mut has_errors = false;
        if let Some(failed) = fields[m.fields().as_range()]
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
            let new_range = self.resolve_fields(fields, m.fields());

            // Check size and align
            ProtocolModel::get_message_meta(self.storage.get_symbol_range(new_range))
                .or_recover(self.diag);

            //TODO: я зачем то передаю range из node, а не new_range. надо это проверить и лучше заменить на другой тип.
            MessageSymbol::new(m.name(), m.fields(), m.direction(), m.kind())
        }
    }

    fn resolve_stream_field(&mut self, node: StreamFieldNode) -> StreamFieldSymbol {
        let kind = match node.kind() {
            StreamFieldTypeNode::Type(name) => {
                StreamFieldType::Type(self.tt.get_type_by_name(name).unwrap())
            }
            StreamFieldTypeNode::StaticArray(array) => {
                let kind = self.tt.get_type_by_name(array.kind()).unwrap();
                let size = Size::try_new(array.size());
                StreamFieldType::StaticArray(ArraySymbol::new(kind, size.or_recover(self.diag)))
            }
            StreamFieldTypeNode::DynamicArray(array) => {
                let kind = self.tt.get_type_by_name(array.kind()).unwrap();
                StreamFieldType::DynamicArray(HeapArraySymbol::new(kind))
            }
        };
        StreamFieldSymbol::new(node.name(), kind)
    }

    /// Returns a range of resolved fields
    fn resolve_stream_fields(
        &mut self,
        fields: &[StreamFieldNode],
        range: FieldsRange,
    ) -> FieldsRange {
        let slice = &fields[range.as_range()];
        for field in slice {
            let symbol = self.resolve_stream_field(*field);
            assert!(self.storage.add_symbol(symbol));
        }

        let len = self.storage.get_symbol_slice::<StreamFieldSymbol>().len() as u16;
        FieldsRange::new(len - u16::from(range.len()), range.len())
    }

    fn resolve_stream_message(&mut self, m: StreamNode, from: &[StreamFieldNode]) -> StreamSymbol {
        let new_range = self.resolve_stream_fields(from, m.fields());
        StreamSymbol::new(m.name(), m.direction(), m.timeout(), m.kind(), new_range)
    }

    fn resolve_messages(
        &mut self,
        m_from: &[MessageNodeType],
        fields: &[FieldNode],
        stream_fields: &[StreamFieldNode],
    ) {
        for m in m_from {
            let kind = match m {
                MessageNodeType::Single(m) => {
                    MessageSymbolType::Single(self.resolve_single_message(*m, m_from, fields))
                }
                MessageNodeType::Stream(m) => {
                    MessageSymbolType::Stream(self.resolve_stream_message(*m, stream_fields))
                }
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
            let meta = TypeMeta::try_new(bytes as u8, 1).or_recover(self.diag);

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
