use compiler_utils::{List, strpool::StringPool};

use crate::{
    model::{
        ResolvedType,
        types::{Access, BuiltinType, NumericType, Protocol, Type},
    },
    table::TypeTable,
    tree::types::ProtocolTree,
};

pub struct Resolver {
    tt: TypeTable,
    pool: StringPool,
}

impl Resolver {
    pub fn new() -> Self {
        let mut tt = TypeTable::default();
        let mut pool = StringPool::default();

        let u8_ = pool.get_id("u8");
        let u16_ = pool.get_id("u16");
        let u32_ = pool.get_id("u32");
        let u64_ = pool.get_id("u64");
        let u128_ = pool.get_id("u128");

        let i8_ = pool.get_id("i8");
        let i16_ = pool.get_id("i16");
        let i32_ = pool.get_id("i32");
        let i64_ = pool.get_id("i64");
        let i128_ = pool.get_id("i128");

        let f32_ = pool.get_id("f32");
        let f64_ = pool.get_id("f64");

        tt.register_type(u8_, Type::Builtin(BuiltinType::Numeric(NumericType::U8)));
        tt.register_type(u16_, Type::Builtin(BuiltinType::Numeric(NumericType::U16)));
        tt.register_type(u32_, Type::Builtin(BuiltinType::Numeric(NumericType::U32)));
        tt.register_type(u64_, Type::Builtin(BuiltinType::Numeric(NumericType::U64)));
        tt.register_type(
            u128_,
            Type::Builtin(BuiltinType::Numeric(NumericType::U128)),
        );

        tt.register_type(i8_, Type::Builtin(BuiltinType::Numeric(NumericType::I8)));
        tt.register_type(i16_, Type::Builtin(BuiltinType::Numeric(NumericType::I16)));
        tt.register_type(i32_, Type::Builtin(BuiltinType::Numeric(NumericType::I32)));
        tt.register_type(i64_, Type::Builtin(BuiltinType::Numeric(NumericType::I64)));
        tt.register_type(
            i128_,
            Type::Builtin(BuiltinType::Numeric(NumericType::I128)),
        );

        tt.register_type(f32_, Type::Builtin(BuiltinType::Numeric(NumericType::F32)));
        tt.register_type(f64_, Type::Builtin(BuiltinType::Numeric(NumericType::F64)));

        Self { tt, pool }
    }

    fn reset_table(&mut self) -> Vec<Type> {
        self.tt.reset()
    }

    pub fn run(&mut self, mut tree: ProtocolTree) -> Protocol {
        let name = self.pool.get_id(&tree.name);
        let version = tree.version.parse::<u8>().unwrap();
        let access = Access::from_str(&tree.access);

        for enum_token in tree.enums.drain(..) {
            let ty = enum_token.resolve(&self.tt, &mut self.pool);
            self.tt.register_type(ty.name(), ty);
        }

        for struct_token in tree.structs.drain(..) {
            let ty = struct_token.resolve(&self.tt, &mut self.pool);
            self.tt.register_type(ty.name(), ty);
        }

        let messages = List::wrap(
            tree.messages
                .drain(..)
                .map(|t| t.resolve(&self.tt, &mut self.pool))
                .collect::<Vec<_>>(),
        );

        let types = List::wrap(self.reset_table());
        Protocol::new(name, version, access, types, messages)
    }
}
