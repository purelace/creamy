use std::{
    collections::{HashMap, HashSet},
    num::ParseIntError,
    str::FromStr,
};

use creamy_protocol_model::{
    Access, BitsetValuesRange, Direction, FieldsRange, FinishedSymbolTable, OptionsRange, Size,
    SymbolTable, TypeId, TypeMeta, VariantsRange, Version,
    constraints::{HEADER_BYTES, MAX_PAYLOAD},
    definition::ProtocolModel,
    error::{Fallback, SemanticError},
    layout::{LayoutCalculator, LayoutItem},
    storage::SymbolStorage,
    symbols::{
        ArraySymbol, BitsetSymbol, BitsetValueSymbol, EnumSymbol, F32_ID, F64_ID, FieldSymbol,
        FieldType, FlagsSymbol, HeapArraySymbol, I8_ID, I16_ID, I32_ID, I64_ID, I128_ID,
        MessageSymbol, MessageSymbolType, Number, ObjectSymbol, OptionSymbol, PrimitiveRepr,
        StreamSymbol, StructSymbol, T_I8_ID, T_I16_ID, T_I32_ID, T_I64_ID, T_I128_ID, T_U8_ID,
        T_U16_ID, T_U32_ID, T_U64_ID, T_U128_ID, Type, U8_ID, U16_ID, U32_ID, U64_ID, U128_ID,
        VariantSymbol, get_builtin_type_name,
    },
};
use creamy_utils::strpool::{StringId, StringPool, StringPoolIntern, StringPoolResolver};
use petgraph::{
    algo::toposort,
    graph::{DiGraph, NodeIndex},
};

use crate::{
    Diagnostics,
    ast::{
        BitsetNode, BitsetValueNode, ComplexType, DefineNode, DefineValueNode, EnumNode, FieldNode,
        FlagsNode, IdentifierNode, MessageNode, MessageNodeType, ModifierNode, NumberNode,
        ObjectNode, OptionNode, StructNode, SyntaxTree, SyntaxTreeBuilder, TypeRefNode,
        VariantNode,
    },
    error::{ProtocolErrorExt, SyntaxError},
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
    storage: SymbolStorage,
    table: SymbolTable,
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
            storage: SymbolStorage::default(),
            table: SymbolTable::new(types),
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

fn resolve_option_list(context: &mut Context, nodes: &[OptionNode<'_>]) -> OptionsRange {
    context.push_scope();

    let offset = context.storage.get_symbol_slice::<OptionSymbol>().len();
    let mut len = 0usize;

    for root in nodes {
        let ident = root.ident.value.intern(&mut context.pool);
        if context.contains_name_in_scope(&root.ident) {
            continue;
        }

        context.storage.add_symbol(OptionSymbol::new(ident));
        len += 1;
    }

    context.pop_scope();
    OptionsRange::new(offset as u16, len as u16)
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
            Type::Flags(FlagsSymbol::new(
                root.ident.value.intern(&mut context.pool),
                options,
            )),
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
    storage: &mut SymbolStorage,
    nodes: &[VariantNode<'_>],
    repr: PrimitiveRepr,
) -> VariantsRange {
    let mut scope = HashSet::new();

    let offset = storage.get_symbol_slice::<VariantSymbol>().len();
    let mut len = 0usize;

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

                storage.add_symbol(VariantSymbol::new(ident, value));
                len += 1;
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

    VariantsRange::new(offset as u16, len as u16)
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

        let symbol = EnumSymbol::new(
            root.ident.value.intern(&mut context.pool),
            repr,
            resolve_variants(
                &mut context.diag,
                &mut context.pool,
                &mut context.storage,
                &root.variants,
                repr,
            ),
        );
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
) -> (u64, BitsetValuesRange) {
    context.push_scope();

    let offset = context
        .storage
        .get_symbol_slice::<BitsetValueSymbol>()
        .len();
    let mut len = 0usize;

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

        context.storage.add_symbol(BitsetValueSymbol::new(
            root.ident.value.intern(&mut context.pool),
            repr,
            width as u8,
        ));

        len += 1;
    }

    let bytes = total_bits.div_ceil(8);
    let range = if bytes > MAX_PAYLOAD as u64 {
        BitsetValuesRange::new(0, 0)
    } else {
        BitsetValuesRange::new(offset as u16, len as u16)
    };

    context.pop_scope();

    (bytes, range)
}

fn resolve_bitsets(context: &mut Context, nodes: &[BitsetNode<'_>]) {
    for root in nodes {
        if context.contains_name_in_scope(&root.ident) {
            continue;
        }

        let (bytes, range) = resolve_bitset_values(context, &root.values);
        let meta = TypeMeta::try_new(bytes, 1).or_recover(&mut context.diag);
        let meta = if meta.size().value() > MAX_PAYLOAD as u64 {
            context.diag.report_err(SemanticError::InvalidSize {
                actual: meta.size().value(),
            });
            TypeMeta::fallback()
        } else {
            meta
        };
        context.table.register_type(
            meta,
            Type::Bitset(BitsetSymbol::new(
                root.ident.value.intern(&mut context.pool),
                range,
            )),
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
    FieldSymbol::new(node.ident.value.intern(pool), kind, false)
}

fn resolve_fields(
    context: &mut Context,
    nodes: &[FieldNode],
    reserved: u8,
    max_align: Option<u8>,
    is_object: bool,
) -> FieldsRange {
    let offset = context.storage.get_symbol_slice::<FieldSymbol>().len();
    let mut len = 0u8;

    let calc = LayoutCalculator::new(
        reserved,
        max_align,
        nodes
            .iter()
            .map(|f| {
                resolve_field(
                    &mut context.diag,
                    &mut context.pool,
                    &mut context.table,
                    f,
                    is_object,
                )
            })
            .collect::<Vec<_>>()
            .into_iter(),
    );

    for (item, _) in calc {
        let symbol = match item {
            LayoutItem::Field(s) => s,
            LayoutItem::Padding { index, size } => FieldSymbol::new(
                context.pool.get_id_or_add(&format!("__padding{index}")),
                FieldType::StaticArray(ArraySymbol::new(T_U8_ID, size)),
                true,
            ),
        };

        context.storage.add_symbol(symbol);
        len += 1;
    }

    FieldsRange::new(offset as u16, len)
}

fn resolve_struct(context: &mut Context, node: &StructNode) {
    let range = resolve_fields(context, &node.fields, 0, None, false);
    let meta = ProtocolModel::get_struct_meta(context.storage.get_symbol_range(range))
        .or_recover(&mut context.diag);
    let s = StructSymbol::new(node.ident.value.intern(&mut context.pool), range);
    context.table.register_type(meta, Type::Struct(s));
}

fn resolve_object(context: &mut Context, node: &ObjectNode) {
    let range = resolve_fields(context, &node.fields, 0, None, true);
    let meta = ProtocolModel::get_struct_meta(context.storage.get_symbol_range(range))
        .or_recover(&mut context.diag);
    let s = ObjectSymbol::new(node.ident.value.intern(&mut context.pool), range);
    context.table.register_type(meta, Type::Object(s));
}

fn resolve_message_modififer(diag: &mut Diagnostics, modifier: &ModifierNode) -> Direction {
    match Direction::from_str(modifier.ident.value) {
        Ok(d) => d,
        Err(_) => todo!(),
    }
}

fn resolve_message(context: &mut Context, node: &MessageNode) {
    match node.kind {
        MessageNodeType::Single => {
            let range = resolve_fields(context, &node.fields, HEADER_BYTES, Some(32), false);

            // Check size and align
            ProtocolModel::get_message_meta(context.storage.get_symbol_range(range))
                .or_recover(&mut context.diag);

            let kind = context.issue_kind();
            context
                .storage
                .add_symbol(MessageSymbolType::Single(MessageSymbol::new(
                    node.ident.value.intern(&mut context.pool),
                    range,
                    resolve_message_modififer(&mut context.diag, &node.modifier),
                    kind,
                )));
        }
        MessageNodeType::Stream => {
            let range = resolve_fields(context, &node.fields, 0, None, true);
            let kind = context.issue_kind();
            context
                .storage
                .add_symbol(MessageSymbolType::Stream(StreamSymbol::new(
                    node.ident.value.intern(&mut context.pool),
                    resolve_message_modififer(&mut context.diag, &node.modifier),
                    4, //TODO: timeout
                    kind,
                    range,
                )));
        }
    }
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

struct Defines {
    name: String,
    version: Version,
    access: Access,
    schema: u32,
}

fn resolve_define_list(context: &mut Context, nodes: &[DefineNode]) -> Defines {
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
                    version = Some(
                        Version::from_str(n.value)
                            .map_err(|_| SyntaxError::InvalidVersionFormat {
                                span: n.span.into(),
                            })
                            .or_recover(&mut context.diag),
                    );
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

    Defines {
        name: name.unwrap(),
        version: version.unwrap(),
        access: access.unwrap(),
        schema: schema.unwrap(),
    }
}

#[derive(Debug)]
pub struct SemanticModel {
    name: String,
    version: Version,
    access: Access,
    schema: u32,
    table: FinishedSymbolTable,
    storage: SymbolStorage,
    pool: StringPool,
}

impl SemanticModel {
    #[must_use]
    pub fn to_protocol_model(mut self) -> ProtocolModel {
        let name = self.name.intern(&mut self.pool);
        ProtocolModel::new(name, self.version, self.access, self.storage, self.table)
    }

    pub fn compile(source: &str) -> Result<Self, Diagnostics> {
        let mut diag = Diagnostics::default();
        let tree: SyntaxTree = SyntaxTreeBuilder::new(source).run(&mut diag);
        if diag.has_errors() {
            return Err(diag);
        }

        let mut context = Context::new(tree.type_count() as u16);
        let defines = resolve_define_list(&mut context, &tree.defines);
        resolve_flags(&mut context, &tree.flags);
        resolve_enums(&mut context, &tree.enums);
        resolve_bitsets(&mut context, &tree.bitsets);
        resolve_complex_types(&tree, &mut context);

        if context.diag.has_errors() {
            return Err(context.diag);
        }

        Ok(Self {
            name: defines.name,
            version: defines.version,
            access: defines.access,
            schema: defines.schema,
            table: context.table.finish(),
            storage: context.storage,
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
    pub const fn pool(&self) -> &StringPool {
        &self.pool
    }

    #[must_use]
    pub fn messages(&self) -> &[MessageSymbolType] {
        self.storage.get_symbol_slice::<MessageSymbolType>()
    }

    #[must_use]
    pub fn types(&self) -> &[Type] {
        self.table.types()
    }

    #[must_use]
    pub const fn table(&self) -> &FinishedSymbolTable {
        &self.table
    }

    #[must_use]
    pub fn fields_range(&self, range: FieldsRange) -> &[FieldSymbol] {
        self.storage.get_symbol_range::<FieldSymbol>(range)
    }

    #[must_use]
    pub fn variants_slice(&self, range: VariantsRange) -> &[VariantSymbol] {
        self.storage.get_symbol_range::<VariantSymbol>(range)
    }

    #[must_use]
    pub fn bvalues_slice(&self, range: BitsetValuesRange) -> &[BitsetValueSymbol] {
        self.storage.get_symbol_range::<BitsetValueSymbol>(range)
    }

    #[must_use]
    pub fn options_slice(&self, range: OptionsRange) -> &[OptionSymbol] {
        self.storage.get_symbol_range::<OptionSymbol>(range)
    }
}

#[cfg(test)]
mod tests {
    const SAMPLE: &str = include_str!("../../../system/system.cmp");

    #[test]
    fn test() {
        match super::SemanticModel::compile(SAMPLE) {
            Ok(model) => {
                dbg!(model);
            }
            Err(d) => {
                d.print(SAMPLE);
                panic!();
            }
        }
    }
}
