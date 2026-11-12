use std::{cell::RefCell, num::NonZeroU8};

use as_guard::AsGuard;
use creamy_protocol_model::{
    BitsetValuesRange, Direction, FieldsRange, MessagesRange, OptionsRange, Size, TypeMeta,
    TypeTable, TypesRange, VariantsRange,
    constraints::{HEADER_BYTES, MAX_PAYLOAD},
    definition::ProtocolModel,
    error::SemanticError,
    layout::{LayoutCalculator, LayoutItem},
    storage::SymbolStorage,
    symbols::{
        ArraySymbol, BitsetSymbol, BitsetValueSymbol, EnumSymbol, F32_ID, F64_ID, FieldSymbol,
        FieldType, FlagsSymbol, GlobalTypesSymbol, GroupSymbol, HeapArraySymbol, I8_ID, I16_ID,
        I32_ID, I64_ID, I128_ID, MessageSymbol, MessageSymbolType,
        NumericSymbol::{self},
        OptionSymbol, PrimitiveRepr, StreamFieldSymbol, StreamFieldType, StreamSymbol,
        StructSymbol, T_U8_ID, Type, U8_ID, U16_ID, U32_ID, U64_ID, U128_ID, VariantSymbol,
    },
};
use creamy_utils::strpool::{StringPool, StringPoolIntern};
use strum::EnumCount;

use crate::{
    diagnostics::Diagnostics,
    error::ProtocolErrorExt,
    tokenizer::tokenize,
    tree::{
        ProtocolTree, StreamFieldNode, StreamFieldTypeNode,
        nodes::{
            BitsetNode, EnumNode, FieldNode, FieldTypeNode, FlagsNode, GlobalTypesNode, GroupNode,
            MessageNode, MessageNodeType, StreamNode, StructNode,
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

    let tree = ProtocolTree::new(tokens, &mut diag);
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
    pool: &'a mut StringPool,
}

impl<'a> DefinitionBuilder<'a> {
    fn new(diag: &'a mut Diagnostics, pool: &'a mut StringPool) -> Self {
        Self {
            storage: SymbolStorage::default(),
            diag,
            pool,
        }
    }

    fn resolve_global_types(
        &mut self,
        global: &GlobalTypesNode,
        tt: &mut TypeTable,
    ) -> GlobalTypesSymbol {
        let mut resolver = Resolver::new(
            self.diag,
            tt,
            &mut self.storage,
            self.pool,
            //FIX: we don't need a group for the global types
            NonZeroU8::new(1).unwrap(),
        );

        resolver.resolve_flags(&global.flags);
        resolver.resolve_bitsets(&global.bitsets);
        resolver.resolve_enums(&global.enums);
        resolver.resolve_structs(&global.structs);

        let offset = NumericSymbol::COUNT as u16;
        let total_len =
            global.flags.len() + global.bitsets.len() + global.enums.len() + global.structs.len();
        GlobalTypesSymbol::new(TypesRange::new(offset, total_len as u16))
    }

    fn resolve_group(
        &mut self,
        tt: &mut TypeTable,
        group: &GroupNode,
        group_id: NonZeroU8,
        offset: u16,
    ) -> GroupSymbol {
        let mut resolver = Resolver::new(self.diag, tt, &mut self.storage, self.pool, group_id);

        resolver.resolve_flags(&group.flags);
        resolver.resolve_bitsets(&group.bitsets);
        resolver.resolve_enums(&group.enums);
        resolver.resolve_structs(&group.structs);

        let before = resolver.storage.count_of::<MessageSymbolType>();
        resolver.resolve_messages(&group.messages);
        let after = resolver.storage.count_of::<MessageSymbolType>();

        let types_count =
            group.flags.len() + group.bitsets.len() + group.enums.len() + group.structs.len();

        GroupSymbol::new(
            group.name.intern(resolver.pool),
            group.access,
            MessagesRange::new(before as u16, after as u16 - before as u16),
            TypesRange::new(offset, types_count as u16),
        )
    }
}

fn run(diag: &mut Diagnostics, pool: &mut StringPool, tree: ProtocolTree) -> ProtocolModel {
    let name = tree.name.intern(pool);
    let mut tt = TypeTable::new(tree.groups.len() as u8, tree.type_count());
    let mut builder = DefinitionBuilder::new(diag, pool);

    let global = builder.resolve_global_types(&tree.global, &mut tt);
    let mut offset = global.types().start() + global.types().len();

    for (index, group) in tree.groups.iter().enumerate() {
        if index >= 255 {
            builder.diag.report_err(SemanticError::TooManyGroups);
            break;
        }

        let group_id = NonZeroU8::new(index.safe_as::<u8>() + 1).expect("Group limit exceeded");
        let symbol = builder.resolve_group(&mut tt, group, group_id, offset);
        offset = symbol.types().start() + global.types().len();
        builder.storage.add_symbol(symbol);
    }

    ProtocolModel::new(name, tree.version, global, builder.storage, tt.finish())
}

struct Resolver<'a> {
    diag: &'a mut Diagnostics,
    tt: &'a mut TypeTable,
    storage: &'a mut SymbolStorage,
    pool: &'a mut StringPool,
    group: NonZeroU8,
    kind: Option<u8>,
    too_many_message_reported: bool,
}

impl<'a> Resolver<'a> {
    const fn new(
        diag: &'a mut Diagnostics,
        tt: &'a mut TypeTable,
        storage: &'a mut SymbolStorage,
        pool: &'a mut StringPool,
        group: NonZeroU8,
    ) -> Self {
        Self {
            diag,
            tt,
            storage,
            pool,
            group,
            kind: Some(0),
            too_many_message_reported: false,
        }
    }

    fn resolve_flags(&mut self, flags: &[FlagsNode]) {
        for f in flags {
            let iter = f
                .options
                .iter()
                .map(|n| OptionSymbol::new(n.name.intern(self.pool)));

            let (start, len) = self.storage.add_symbol_from_iter(iter);

            let bits = f.options.len();
            let bytes = bits.div_ceil(8);
            let meta = match bytes {
                1 => TypeMeta::S1B_A1B,
                2 => TypeMeta::S2B_A2B,
                3 | 4 => TypeMeta::S4B_A4B,
                5..=8 => TypeMeta::S8B_A8B,
                9..=16 => TypeMeta::S16B_A16B,
                _ => unreachable!(),
            };

            let symbol = FlagsSymbol::new(
                f.name.intern(self.pool),
                OptionsRange::new(start as u16, len as u16),
            );
            self.tt.register_type(self.group, meta, Type::Flags(symbol));
        }
    }

    fn resolve_bitsets(&mut self, from: &[BitsetNode]) {
        const MAX_BITS: usize = MAX_PAYLOAD * 8;
        for f in from {
            let mut has_errors = false;
            let mut total_bits = 0;
            let mut bv_symbols = Vec::new();
            for value in &f.values {
                total_bits += value.bits;
                if value.bits > MAX_BITS {
                    has_errors = true;
                    break;
                }
                if total_bits > MAX_BITS {
                    has_errors = true;
                    break;
                }

                let repr = self
                    .tt
                    .get_type_by_name(value.repr.intern(self.pool))
                    .unwrap();

                bv_symbols.push(BitsetValueSymbol::new(
                    value.name.intern(self.pool),
                    repr,
                    value.bits as u8,
                ));
            }

            let (start, len) = self.storage.add_symbol_from_iter(bv_symbols);

            let bytes = total_bits.div_ceil(8);
            let meta = TypeMeta::try_new(bytes as u8, 1).or_recover(self.diag);

            let range = if has_errors {
                BitsetValuesRange::new(0, 0)
            } else {
                BitsetValuesRange::new(start as u16, len as u16)
            };

            let symbol = BitsetSymbol::new(f.name.intern(self.pool), range);

            self.tt
                .register_type(self.group, meta, Type::Bitset(symbol));
        }
    }

    fn resolve_enums(&mut self, enums: &[EnumNode]) {
        for e in enums {
            let result = PrimitiveRepr::try_from(e.repr.as_str());
            let repr = match result {
                Ok(repr) => repr,
                Err(err) => {
                    self.diag.report_err(err);
                    PrimitiveRepr::U32
                }
            };

            //TODO: check duplicates

            let iter = e.variants.iter().filter_map(|n| {
                let value = n.value;
                if repr.is_valid_value(value) {
                    Some(VariantSymbol::new(n.name.intern(self.pool), value))
                } else {
                    self.diag
                        .report_err(SemanticError::EnumVariantValueOutOfRange {
                            value,
                            min: repr.get_min(),
                            max: repr.get_max(),
                        });
                    None
                }
            });

            let (start, len) = self.storage.add_symbol_from_iter(iter);

            let symbol = EnumSymbol::new(
                e.name.intern(self.pool),
                repr,
                VariantsRange::new(start as u16, len as u16),
            );
            let meta = symbol.meta().or_recover(self.diag);
            self.tt.register_type(self.group, meta, Type::Enum(symbol));
        }
    }

    fn resolve_field(
        diag: &mut Diagnostics,
        pool: &mut StringPool,
        tt: &TypeTable,
        node: &FieldNode,
    ) -> FieldSymbol {
        let kind = match &node.kind {
            FieldTypeNode::Type(name) => {
                FieldType::Type(tt.get_type_by_name(name.intern(pool)).unwrap())
            }
            FieldTypeNode::Array(array) => {
                let kind = tt.get_type_by_name(array.kind.intern(pool)).unwrap();
                let size = Size::try_new(array.size);
                FieldType::Array(ArraySymbol::new(kind, size.or_recover(diag)))
            }
        };
        FieldSymbol::new(node.name.intern(pool), kind, false)
    }

    /// Returns a range of resolved fields
    fn resolve_fields(
        &mut self,
        fields: &[FieldNode],
        reserved: u8,
        max_align: Option<u8>,
    ) -> FieldsRange {
        let Self {
            diag,
            tt,
            storage,
            pool,
            ..
        } = self;

        let slice_len_before = storage.get_symbol_slice::<FieldSymbol>().len() as u16;

        let calc = LayoutCalculator::new(
            reserved,
            max_align,
            fields
                .iter()
                .map(|f| Self::resolve_field(diag, pool, tt, f))
                .collect::<Vec<_>>()
                .into_iter(),
        );

        for (item, _) in calc {
            let symbol = match item {
                LayoutItem::Field(s) => s,
                LayoutItem::Padding { index, size } => FieldSymbol::new(
                    pool.get_id_or_add(&format!("__padding{index}")),
                    FieldType::Array(ArraySymbol::new(T_U8_ID, size)),
                    true,
                ),
            };

            storage.add_symbol(symbol);
        }

        let slice_len_after = storage.get_symbol_slice::<FieldSymbol>().len() as u16;
        let len = slice_len_after - slice_len_before;
        FieldsRange::new(slice_len_before, len as u8)
    }

    fn resolve_structs(&mut self, structs: &[StructNode]) {
        let mut to_resolve = structs.len();
        // Устанавливаем true чтобы пройти первую итерацию цикла
        let mut is_any_resolved = true;
        while to_resolve != 0 {
            for s in structs {
                if self.tt.contains_name(s.name.intern(self.pool)) {
                    continue;
                }

                let mut has_errors = false;

                if let Some(failed) = s
                    .fields
                    .iter()
                    .find(|f| !self.tt.contains_name(f.kind.type_name().intern(self.pool)))
                {
                    if is_any_resolved {
                        is_any_resolved = false;
                        continue;
                    }

                    let kind = failed.kind.type_name();
                    let error = if s.name.as_str() == kind {
                        SemanticError::SelfReference(s.name.to_string())
                    //} else if messages.iter().any(|m| m.name() == kind) {
                    //    ProtocolError::MessageReference(name.to_string())
                    } else {
                        SemanticError::CannotResolveTypeFieldNotFound {
                            from: s.name.to_string(),
                            kind: kind.to_string(),
                        }
                    };
                    self.diag.report_err(error);
                    has_errors = true;
                }

                if has_errors {
                    self.tt.register_type(
                        self.group,
                        TypeMeta::S1B_A1B,
                        Type::Struct(StructSymbol::new(
                            s.name.intern(self.pool),
                            FieldsRange::new(0, 0),
                        )),
                    );
                } else {
                    let new_range = self.resolve_fields(&s.fields, 0, None);
                    let meta =
                        ProtocolModel::get_struct_meta(self.storage.get_symbol_range(new_range))
                            .or_recover(self.diag);
                    let s = StructSymbol::new(s.name.intern(self.pool), new_range);

                    self.tt.register_type(self.group, meta, Type::Struct(s));
                }

                to_resolve -= 1;
                is_any_resolved = false;
            }
        }
    }

    fn issue_kind(&mut self) -> u8 {
        if let Some(current) = self.kind {
            self.kind = current.checked_add(1);
            current
        } else {
            if !self.too_many_message_reported {
                self.diag.report_err(SemanticError::TooManyMessages);
                self.too_many_message_reported = true;
            }
            0
        }
    }
    fn resolve_single_message(
        &mut self,
        m: &MessageNode,
        messages: &[MessageNodeType],
    ) -> MessageSymbol {
        let mut has_errors = false;

        if let Some(failed) = m
            .fields
            .iter()
            .find(|f| !self.tt.contains_name(f.kind.type_name().intern(self.pool)))
        {
            let kind = failed.kind.type_name();
            let error = if m.name.as_str() == kind {
                SemanticError::SelfReference(m.name.to_string())
            } else if messages.iter().any(|m| m.name() == kind) {
                SemanticError::MessageReference(m.name.to_string())
            } else {
                SemanticError::CannotResolveTypeFieldNotFound {
                    from: m.name.to_string(),
                    kind: kind.to_string(),
                }
            };
            self.diag.report_err(error);
            has_errors = true;
        }

        if has_errors {
            MessageSymbol::new(
                m.name.intern(self.pool),
                FieldsRange::new(0, 0),
                Direction::Incoming,
                0,
            )
        } else {
            let new_range = self.resolve_fields(&m.fields, HEADER_BYTES, Some(32));

            // Check size and align
            ProtocolModel::get_message_meta(self.storage.get_symbol_range(new_range))
                .or_recover(self.diag);

            MessageSymbol::new(
                m.name.intern(self.pool),
                new_range,
                m.direction,
                self.issue_kind(),
            )
        }
    }

    fn resolve_stream_field(
        diag: &mut Diagnostics,
        pool: &mut StringPool,
        tt: &TypeTable,
        node: &StreamFieldNode,
    ) -> StreamFieldSymbol {
        let kind = match &node.kind {
            StreamFieldTypeNode::Type(name) => {
                StreamFieldType::Type(tt.get_type_by_name(name.intern(pool)).unwrap())
            }
            StreamFieldTypeNode::StaticArray(array) => {
                let kind = tt.get_type_by_name(array.kind.intern(pool)).unwrap();
                let size = Size::try_new(array.size);
                StreamFieldType::StaticArray(ArraySymbol::new(kind, size.or_recover(diag)))
            }
            StreamFieldTypeNode::DynamicArray(array) => {
                let kind = tt.get_type_by_name(array.kind.intern(pool)).unwrap();
                StreamFieldType::DynamicArray(HeapArraySymbol::new(kind))
            }
        };
        StreamFieldSymbol::new(node.name.intern(pool), kind, false)
    }

    /// Returns a range of resolved fields
    fn resolve_stream_fields(&mut self, fields: &[StreamFieldNode]) -> FieldsRange {
        let Self {
            diag,
            tt,
            storage,
            pool,
            ..
        } = self;

        let slice_len_before = storage.get_symbol_slice::<StreamFieldSymbol>().len() as u16;
        let calc = LayoutCalculator::new(
            0,
            None,
            fields
                .iter()
                .map(|f| Self::resolve_stream_field(diag, pool, tt, f))
                .collect::<Vec<_>>()
                .into_iter(),
        );

        for (item, _) in calc {
            let symbol = match item {
                LayoutItem::Field(s) => s,
                LayoutItem::Padding { index, size } => StreamFieldSymbol::new(
                    pool.get_id_or_add(&format!("__padding{index}")),
                    StreamFieldType::StaticArray(ArraySymbol::new(T_U8_ID, size)),
                    true,
                ),
            };

            storage.add_symbol(symbol);
        }

        let slice_len_after = storage.get_symbol_slice::<StreamFieldSymbol>().len() as u16;
        let len = slice_len_after - slice_len_before;
        FieldsRange::new(slice_len_before, len as u8)
    }

    fn resolve_stream_message(&mut self, m: &StreamNode) -> StreamSymbol {
        StreamSymbol::new(
            m.name.intern(self.pool),
            m.direction,
            m.timeout,
            self.issue_kind(),
            self.resolve_stream_fields(&m.fields),
        )
    }

    fn resolve_messages(&mut self, messages: &[MessageNodeType]) {
        for m in messages {
            let kind = match m {
                MessageNodeType::Single(m) => {
                    MessageSymbolType::Single(self.resolve_single_message(m, messages))
                }
                MessageNodeType::Stream(m) => {
                    MessageSymbolType::Stream(self.resolve_stream_message(m))
                }
            };

            self.storage.add_symbol(kind);
        }
    }
}
