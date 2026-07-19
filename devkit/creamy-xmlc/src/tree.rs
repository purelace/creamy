mod bitset;
mod enums;
mod flags;
mod message;
mod structs;

use semver::Version;

use crate::{
    Access, Diagnostics, StringPoolIntern,
    error::{AstError, Fallback},
    nodes::{
        BValueNode, BitsetNode, EnumNode, FieldNode, FieldTypeNode, FlagsNode, GroupNode,
        MessageNodeType, OptionNode, StreamPayloadFieldNode, StructNode, VariantNode,
    },
    tokenizer::{Identifier, Token},
    tree::{
        bitset::{BValueParser, BitsetParser},
        enums::{EnumParser, VariantParser},
        flags::{FlagsParser, OptionParser},
        message::{MessageParser, StreamPayloadFieldParser},
        structs::StructParser,
    },
    utils::{
        BValuesRange, BitsetsRange, BoundedVec, EnumsRange, FieldsRange, FlagsRange, MessagesRange,
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

    const fn build_bvalues(&mut self) -> BValuesRange {
        let range = BValuesRange::new(self.start as u16, self.len as u16);
        self.build();
        range
    }
}

#[derive(Debug)]
pub struct ProtocolTree {
    pub name: StringId,
    pub version: Version,

    pub global: GroupNode,

    pub groups: BoundedVec<GroupNode>,

    pub options: BoundedVec<OptionNode>,
    pub bvalues: BoundedVec<BValueNode>,
    pub fields: BoundedVec<FieldNode>,
    pub variants: BoundedVec<VariantNode>,
    pub payload: BoundedVec<StreamPayloadFieldNode>,

    pub messages: BoundedVec<MessageNodeType>,
    pub structs: BoundedVec<StructNode>,
    pub enums: BoundedVec<EnumNode>,
    pub flags: BoundedVec<FlagsNode>,
    pub bitsets: BoundedVec<BitsetNode>,
}

#[inline]
const fn token_field(t: &Token) -> bool {
    matches!(t, Token::Field { .. })
}

impl ProtocolTree {
    #[allow(clippy::too_many_lines)]
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

        let mut global_message_builder = RangeBuilder::default();
        let mut global_struct_builder = RangeBuilder::default();
        let mut global_enum_builder = RangeBuilder::default();
        let mut global_flag_builder = RangeBuilder::default();
        let mut global_bitset_builder = RangeBuilder::default();

        let mut groups = BoundedVec::new();
        let mut group: Option<(StringId, Access)> = None;
        let mut group_err = false;

        let mut ctx = ParserContext::new(pool, diagnostics);
        let mut messages = MessageParser::default();
        let mut structs = StructParser::default();
        let mut enums = EnumParser::default();
        let mut flags = FlagsParser::default();
        let mut bitsets = BitsetParser::default();

        let mut iter = tokens.drain(..).peekable();
        while let Some(token) = iter.next() {
            match token {
                Token::Group {
                    name,
                    access,
                    span: _,
                } => {
                    if let Some((name, access)) = group.take() {
                        let messages = messages.builder().build_messages();
                        let structs = structs.builder().build_structs();
                        let enums = enums.builder().build_enums();
                        let flags = flags.builder().build_flags();
                        let bitsets = bitsets.builder().build_bitsets();
                        if !groups.push(GroupNode::new(
                            name, access, messages, structs, enums, flags, bitsets,
                        )) && !group_err
                        {
                            group_err = true;
                            ctx.diag.report_err(AstError::TooManyGroups);
                        }
                    } else {
                        //reset if group_name == None because we have global group
                        messages.builder().build();
                        structs.builder().build();
                        enums.builder().build();
                        flags.builder().build();
                        bitsets.builder().build();
                    }

                    group = Some((name.intern(ctx.pool), access));
                }
                Token::Message {
                    kind,
                    name,
                    span: _,
                } => {
                    messages.parse_single(kind, name, &mut iter, &mut ctx);
                }
                Token::Struct { name, span: _ } => {
                    structs.parse_struct(name, &mut iter, &mut ctx);
                }
                Token::Enum {
                    name,
                    repr,
                    span: _,
                } => {
                    enums.parse_enum(name, repr, &mut iter, &mut ctx);
                }
                Token::Flags { name, span: _ } => {
                    flags.parse_flags(name, &mut iter, &mut ctx);
                }
                Token::Bitset { name, span: _ } => {
                    bitsets.parse_bitset(name, &mut iter, &mut ctx);
                }
                Token::Stream {
                    kind,
                    name,
                    timeout,
                    span: _,
                } => {
                    messages.parse_stream(kind, name, timeout, &mut iter, &mut ctx);
                }
                other => {
                    ctx.diag
                        .report_err(AstError::UnexpectedToken { span: other.span() });
                }
            }
        }

        if let Some((name, access)) = group.take() {
            let messages = messages.builder().build_messages();
            let structs = structs.builder().build_structs();
            let enums = enums.builder().build_enums();
            let flags = flags.builder().build_flags();
            let bitsets = bitsets.builder().build_bitsets();
            if !groups.push(GroupNode::new(
                name, access, messages, structs, enums, flags, bitsets,
            )) && !group_err
            {
                ctx.diag.report_err(AstError::TooManyGroups);
            }
        }

        ProtocolTree {
            name: ctx.pool.get_id_or_add(&name),
            version,

            global: GroupNode::new(
                ctx.pool.get_id_or_add(&name),
                Access::Public, //TODO fix
                global_message_builder.build_messages(),
                global_struct_builder.build_structs(),
                global_enum_builder.build_enums(),
                global_flag_builder.build_flags(),
                global_bitset_builder.build_bitsets(),
            ),
            groups,

            options: ctx.option.take(),
            bvalues: ctx.bvalue.take(),
            fields: ctx.field.take(),
            variants: ctx.variant.take(),

            messages: messages.take(),

            structs: structs.take(),
            enums: enums.take(),
            flags: flags.take(),
            bitsets: bitsets.take(),
            payload: ctx.payload.take(),
        }
    }

    /// Значение не может превышать [``crate::constraints::MAX_TYPE_COUNT``]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn type_count(&self) -> u16 {
        (self.enums.len() + self.structs.len() + self.flags.len() + self.bitsets.len()) as u16
    }
}

#[macro_export]
macro_rules! push_node {
    (
        to: $vec:expr, node: $node:expr, flag: $err_flag:expr, $diag:expr, $err_type:expr) => {
        if !$vec.push($node) && !$err_flag {
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
            vec: $crate::utils::BoundedVec<$type>,
            has_error: bool,
            builder: $crate::tree::RangeBuilder,
        }

        impl $name {
            #[allow(unused)]
            pub fn $parse_all(
                &mut self,
                diag: &mut $crate::Diagnostics,
                pool: &mut $crate::utils::strpool::StringPool,
                iter: &mut core::iter::Peekable<std::vec::Drain<$crate::tokenizer::Token>>,
            ) -> $ret {
                while let Some($token_path { $($f_name,)* .. }) = iter.next_if($predicate) {
                    let node = {
                        let $pool_name = &mut *pool;
                        $(let $c_arg = $f_name;)*
                        $closure
                    };

                    $crate::push_node! {
                        to: self.vec,
                        node: node,
                        flag: self.has_error,
                        diag,
                        $error
                    }
                    self.builder.next();
                }

                self.builder.$builder()
            }

            pub fn take(self) -> $crate::utils::BoundedVec<$type> {
                self.vec
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
            vec: $crate::utils::BoundedVec<$type>,
            has_error: bool,
            builder: $crate::tree::RangeBuilder,
        }

        impl $name {
            pub fn $t_parse_method(
                &mut self,
                $($arg : $arg_type,)* // Объявление аргументов в методе
                iter: &mut core::iter::Peekable<std::vec::Drain<$crate::tokenizer::Token>>,
                ctx: &mut $crate::tree::ParserContext,
            ) {
                // Вызов вложенного парсера
                let range = ctx.$misc.$m_parse(ctx.diag, ctx.pool, iter);

                // Переменные для конструктора (ctx и range доступны внутри выражения)
                let node = {
                    let $ctx_name = &mut *ctx;
                    let $range_name = range;
                    $(let $c_arg = $arg;)* // Привязываем аргументы метода к именам в "замыкании"
                    $closure
                };

                self.builder.next();

                $crate::push_node!(
                    to: self.vec,
                    node: node,
                    flag: self.has_error,
                    ctx.diag,
                    $error_type
                );
            }

            pub const fn builder(&mut self) -> &mut $crate::tree::RangeBuilder {
                &mut self.builder
            }

            pub fn take(self) -> $crate::utils::BoundedVec<$type> {
                self.vec
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
