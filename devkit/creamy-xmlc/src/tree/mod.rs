mod bitset;
mod enums;
mod flags;
mod message;
pub mod nodes;
pub mod storage;
mod structs;

use nodes::{
    BitsetNode, BitsetValueNode, EnumNode, FieldNode, FieldTypeNode, FlagsNode, GlobalTypesNode,
    GroupNode, MessageNodeType, OptionNode, StreamPayloadFieldNode, StructNode, VariantNode,
};
use semver::Version;

use crate::{
    Access, Diagnostics, StringPoolIntern,
    error::{AstError, Fallback},
    tokenizer::{Identifier, Token},
    tree::{
        bitset::{BValueParser, BitsetParser},
        enums::{EnumParser, VariantParser},
        flags::{FlagsParser, OptionParser},
        message::{MessageParser, StreamPayloadFieldParser},
        storage::NodeStorage,
        structs::StructParser,
    },
    utils::{
        BitsetValuesRange, BitsetsRange, EnumsRange, FieldsRange, FlagsRange, MessagesRange,
        OptionsRange, StructsRange, VariantsRange,
        strpool::{StringId, StringPool},
    },
};

#[derive(Default)]
pub(super) struct RangeBuilder {
    start: usize,
    len: usize,
}

impl RangeBuilder {
    const fn next(&mut self) {
        self.len += 1;
    }

    const fn build(&mut self) {
        self.start += self.len;
        self.len = 0;
    }

    const fn build_enums(&mut self) -> EnumsRange {
        let range = EnumsRange::new(self.start as u16, self.len as u16);
        self.build();
        range
    }

    const fn build_structs(&mut self) -> StructsRange {
        let range = StructsRange::new(self.start as u16, self.len as u16);
        self.build();
        range
    }

    const fn build_messages(&mut self) -> MessagesRange {
        let range = MessagesRange::new(self.start as u16, self.len as u16);
        self.build();
        range
    }

    const fn build_flags(&mut self) -> FlagsRange {
        let range = FlagsRange::new(self.start as u16, self.len as u16);
        self.build();
        range
    }

    const fn build_bitsets(&mut self) -> BitsetsRange {
        let range = BitsetsRange::new(self.start as u16, self.len as u16);
        self.build();
        range
    }

    const fn build_fields(&mut self) -> FieldsRange {
        let range = FieldsRange::new(self.start as u16, self.len as u8);
        self.build();
        range
    }

    const fn build_variants(&mut self) -> VariantsRange {
        let range = VariantsRange::new(self.start as u16, self.len as u16);
        self.build();
        range
    }

    const fn build_options(&mut self) -> OptionsRange {
        let range = OptionsRange::new(self.start as u16, self.len as u16);
        self.build();
        range
    }

    const fn build_bvalues(&mut self) -> BitsetValuesRange {
        let range = BitsetValuesRange::new(self.start as u16, self.len as u16);
        self.build();
        range
    }
}

//TODO: rework
fn get_node_storage() -> NodeStorage {
    let mut storage = NodeStorage::default();
    storage.register_node::<GroupNode>();
    storage.register_node::<OptionNode>();
    storage.register_node::<BitsetValueNode>();
    storage.register_node::<FieldNode>();
    storage.register_node::<VariantNode>();
    storage.register_node::<StreamPayloadFieldNode>();
    storage.register_node::<MessageNodeType>();
    storage.register_node::<StructNode>();
    storage.register_node::<EnumNode>();
    storage.register_node::<FlagsNode>();
    storage.register_node::<BitsetNode>();
    storage
}

#[derive(Debug)]
pub struct ProtocolTree {
    pub name: StringId,
    pub version: Version,
    pub global: GlobalTypesNode,
    pub storage: NodeStorage,
}

#[inline]
const fn token_field(t: &Token) -> bool {
    matches!(t, Token::Field { .. })
}

#[derive(Debug, PartialEq, Eq)]
enum AnalysisState {
    //Extension,
    Protocol,
    Global,
    Group { name: StringId, access: Access },
}

impl ProtocolTree {
    pub fn new(
        mut tokens: Vec<Token>,
        pool: &mut StringPool,
        diagnostics: &mut Diagnostics,
    ) -> Self {
        let (name, version) = if let Token::Protocol {
            name,
            version,
            span: _,
        } = tokens.remove(0)
        {
            (name, version)
        } else {
            diagnostics.report_err(AstError::MissingProtocolToken);
            (Identifier::fallback(), Version::fallback())
        };

        let mut storage = get_node_storage();
        let mut global_types = GlobalTypesNode::default();

        let mut group_err = false;

        let mut ctx = ParserContext::new(pool, diagnostics);
        let mut messages = MessageParser::default();
        let mut structs = StructParser::default();
        let mut enums = EnumParser::default();
        let mut flags = FlagsParser::default();
        let mut bitsets = BitsetParser::default();

        let mut iter = tokens.drain(..).peekable();
        let mut state = AnalysisState::Protocol;
        loop {
            match state {
                AnalysisState::Protocol => match iter.peek() {
                    Some(&Token::Group {
                        name,
                        access,
                        span: _,
                    }) => {
                        let _ = iter.next().unwrap();
                        // применить сдвиги к счетчикам
                        messages.builder().build();
                        structs.builder().build();
                        enums.builder().build();
                        flags.builder().build();
                        bitsets.builder().build();
                        state = AnalysisState::Group {
                            name: name.intern(ctx.pool),
                            access,
                        };
                    }
                    Some(&Token::Global { .. }) => {
                        let _ = iter.next().unwrap();
                        // применить сдвиги к счетчикам
                        structs.builder().build();
                        enums.builder().build();
                        flags.builder().build();
                        bitsets.builder().build();
                        state = AnalysisState::Global;
                    }
                    Some(other) => {
                        let span = other.span();
                        let _ = iter.next().unwrap();
                        ctx.diag.report_err(AstError::UnexpectedToken { span });
                    }
                    None => {
                        break;
                    }
                },
                AnalysisState::Global => match iter.peek() {
                    Some(&Token::Struct { name, .. }) => {
                        let _ = iter.next().unwrap();
                        structs.parse_struct(name, &mut storage, &mut iter, &mut ctx);
                    }
                    Some(&Token::Enum { name, repr, .. }) => {
                        let _ = iter.next().unwrap();
                        enums.parse_enum(name, repr, &mut storage, &mut iter, &mut ctx);
                    }
                    Some(&Token::Flags { name, .. }) => {
                        let _ = iter.next().unwrap();
                        flags.parse_flags(name, &mut storage, &mut iter, &mut ctx);
                    }
                    Some(&Token::Bitset { name, .. }) => {
                        let _ = iter.next().unwrap();
                        bitsets.parse_bitset(name, &mut storage, &mut iter, &mut ctx);
                    }
                    _ => {
                        global_types = GlobalTypesNode::new(
                            structs.builder().build_structs(),
                            enums.builder().build_enums(),
                            flags.builder().build_flags(),
                            bitsets.builder().build_bitsets(),
                        );
                        state = AnalysisState::Protocol;
                    }
                },
                AnalysisState::Group { name, access } => match iter.peek() {
                    Some(&Token::Message {
                        kind,
                        name,
                        direction,
                        ..
                    }) => {
                        let _ = iter.next().unwrap();
                        messages.parse_single(
                            kind,
                            name,
                            direction,
                            &mut storage,
                            &mut iter,
                            &mut ctx,
                        );
                    }
                    Some(&Token::Stream {
                        kind,
                        name,
                        timeout,
                        direction,
                        ..
                    }) => {
                        let _ = iter.next().unwrap();
                        messages.parse_stream(
                            kind,
                            name,
                            direction,
                            timeout,
                            &mut storage,
                            &mut iter,
                            &mut ctx,
                        );
                    }

                    Some(&Token::Struct { name, .. }) => {
                        let _ = iter.next().unwrap();
                        structs.parse_struct(name, &mut storage, &mut iter, &mut ctx);
                    }
                    Some(&Token::Enum { name, repr, .. }) => {
                        let _ = iter.next().unwrap();
                        enums.parse_enum(name, repr, &mut storage, &mut iter, &mut ctx);
                    }
                    Some(&Token::Flags { name, .. }) => {
                        let _ = iter.next().unwrap();
                        flags.parse_flags(name, &mut storage, &mut iter, &mut ctx);
                    }
                    Some(&Token::Bitset { name, .. }) => {
                        let _ = iter.next().unwrap();
                        bitsets.parse_bitset(name, &mut storage, &mut iter, &mut ctx);
                    }
                    _ => {
                        let node = GroupNode::new(
                            name,
                            access,
                            messages.builder().build_messages(),
                            structs.builder().build_structs(),
                            enums.builder().build_enums(),
                            flags.builder().build_flags(),
                            bitsets.builder().build_bitsets(),
                        );

                        if !storage.add_node(node) && !group_err {
                            group_err = true;
                            ctx.diag.report_err(AstError::TooManyGroups);
                        }

                        state = AnalysisState::Protocol;
                    }
                },
            }
        }

        assert_eq!(state, AnalysisState::Protocol);

        ProtocolTree {
            name: ctx.pool.get_id_or_add(&name),
            version,
            global: global_types,
            storage,
        }
    }

    /// Значение не может превышать [``crate::constraints::MAX_TYPE_COUNT``]
    #[allow(clippy::cast_possible_truncation)]
    pub fn type_count(&self) -> u16 {
        self.storage.type_count() as u16
    }
}

#[macro_export]
macro_rules! push_node {
    (
        to: $storage:expr, node: $node:expr, flag: $err_flag:expr, $diag:expr, $err_type:expr) => {
        if !$storage.add_node($node) && !$err_flag {
            $err_flag = true;
            $diag.report_err($err_type);
        }
    };
}

#[macro_export]
macro_rules! define_misc_parser {
    (
        name:    $name: ident,
        type:    $type: ty,
        return:  $ret:  ty,
        error:   $error: path,
        fn:      $parse_all: ident,
        builder: $builder: ident,
        token:   $token_path: path,
        fields:  [ $($f_name:ident),* $(,)? ],
        predicate: $predicate: expr,
        ctor:    |$pool_name:ident, $($c_arg:ident),*| $closure:expr
    ) => {
        #[derive(Default)]
        pub(crate) struct $name {
            has_error: bool,
            builder: $crate::tree::RangeBuilder,
        }

        impl $name {
            #[allow(unused)]
            pub fn $parse_all(
                &mut self,
                diag: &mut $crate::Diagnostics,
                pool: &mut $crate::utils::strpool::StringPool,
                storage: &mut $crate::tree::storage::NodeStorage,
                iter: &mut core::iter::Peekable<std::vec::Drain<$crate::tokenizer::Token>>,
            ) -> $ret {
                while let Some($token_path { $($f_name,)* .. }) = iter.next_if($predicate) {
                    let node = {
                        let $pool_name = &mut *pool;
                        $(let $c_arg = $f_name;)*
                        $closure
                    };

                    $crate::push_node! {
                        to: storage,
                        node: node,
                        flag: self.has_error,
                        diag,
                        $error
                    }
                    self.builder.next();
                }

                self.builder.$builder()
            }
        }
    };
}

#[macro_export]
macro_rules! define_toplevel_parser {
    (
        name:       $name:ident,
        type:       $type:ty,
        error:      $error_type:path,
        top_parse:  $t_parse_method:ident,
        misc_parse: $misc:ident.$m_parse:ident,
        args:       [ $($arg:ident : $arg_type:ty),* $(,)? ], // Исправлено на :ty
        ctor:       |$ctx_name:ident, $range_name:ident, $($c_arg:ident),*| $closure:expr
    ) => {
        #[derive(Default)]
        pub(crate) struct $name {
            has_error: bool,
            builder: $crate::tree::RangeBuilder,
        }

        impl $name {
            pub fn $t_parse_method(
                &mut self,
                $($arg : $arg_type,)* // Объявление аргументов в методе
                storage: &mut $crate::tree::storage::NodeStorage,
                iter: &mut core::iter::Peekable<std::vec::Drain<$crate::tokenizer::Token>>,
                ctx: &mut $crate::tree::ParserContext,
            ) {
                // Вызов вложенного парсера
                let range = ctx.$misc.$m_parse(ctx.diag, ctx.pool, storage, iter);

                // Переменные для конструктора (ctx и range доступны внутри выражения)
                let node = {
                    let $ctx_name = &mut *ctx;
                    let $range_name = range;
                    $(let $c_arg = $arg;)* // Привязываем аргументы метода к именам в "замыкании"
                    $closure
                };

                self.builder.next();

                $crate::push_node!(
                    to: storage,
                    node: node,
                    flag: self.has_error,
                    ctx.diag,
                    $error_type
                );
            }

            pub const fn builder(&mut self) -> &mut $crate::tree::RangeBuilder {
                &mut self.builder
            }
        }
    };
}

pub(super) struct ParserContext<'a> {
    pool: &'a mut StringPool,
    diag: &'a mut Diagnostics,
    field: FieldParser,
    variant: VariantParser,
    option: OptionParser,
    bvalue: BValueParser,
    payload: StreamPayloadFieldParser,
}

impl<'a> ParserContext<'a> {
    fn new(pool: &'a mut StringPool, diag: &'a mut Diagnostics) -> Self {
        Self {
            pool,
            diag,
            field: FieldParser::default(),
            variant: VariantParser::default(),
            option: OptionParser::default(),
            bvalue: BValueParser::default(),
            payload: StreamPayloadFieldParser::default(),
        }
    }
}

define_misc_parser! {
    name:      FieldParser,
    type:      FieldNode,
    return:    FieldsRange,
    error:     AstError::TooManyFields,
    fn:        parse_fields,
    builder:   build_fields,
    token:     Token::Field,
    fields:    [ name, kind, span ],
    predicate: token_field,
    ctor:      |pool, name, kind, span| {
        FieldNode::new(name.intern(pool), FieldTypeNode::new(kind, pool))
    }
}
