pub mod proxy;

use std::borrow::Cow;

use creamy_manifest::Manifest;
use creamy_xmlc::{
    ProtocolDefinition, StringPoolResolver, compile,
    constraints::{HEADER_BYTES, MAX_PAYLOAD},
    model::symbols::{BitsetSymbol, MessageSymbolType, Type},
    utils::strpool::StringPool,
};

use self::proxy::{
    EnrichedBitsetSymbol, EnrichedEnumSymbol, EnrichedFieldSymbol, EnrichedFieldType,
    EnrichedFlagsSymbol, FieldList, OptionList, ResolvedVariant,
};

#[derive(Clone)]
pub struct EnrichedStructSymbol<'s, I>
where
    I: Iterator<Item = EnrichedFieldSymbol<'s>>,
{
    pub name: &'s str,
    pub fields: I,
}

pub trait CodeGenerator {
    fn start_group(&mut self, group: &str);
    fn generate_single_message<'s, I>(&mut self, symbol: EnrichedSingleMessageSymbol<'s, I>)
    where
        I: Iterator<Item = EnrichedFieldSymbol<'s>>;

    fn generate_stream_message<'s, F, I>(&mut self, symbol: EnrichedStreamMessageSymbol<'s, F, I>)
    where
        F: Iterator<Item = EnrichedFieldSymbol<'s>>,
        I: Iterator<Item = EnrichedFieldSymbol<'s>>;

    fn generate_bitset(&mut self, symbol: BitsetSymbol);

    fn generate_flags<'s, I>(&mut self, symbol: EnrichedFlagsSymbol<'s, I>)
    where
        I: Iterator<Item = &'s str>;

    fn generate_enum<'s, I>(&mut self, symbol: EnrichedEnumSymbol<'s, I>)
    where
        I: Iterator<Item = ResolvedVariant<'s>>;

    fn generate_struct<'s, I>(&mut self, symbol: EnrichedStructSymbol<'s, I>)
    where
        I: Iterator<Item = EnrichedFieldSymbol<'s>>;

    fn end_group(&mut self);
}

pub struct Codegen<'s, G: CodeGenerator> {
    content: Cow<'s, str>,
    generator: G,
}

impl<'s, G: CodeGenerator> Codegen<'s, G> {
    pub fn new(content: impl Into<Cow<'s, str>>, generator: G) -> Self {
        let content = content.into();
        Self { content, generator }
    }

    pub fn with_manifest(&mut self, manifest: &str) {
        let (args, manifest) = Manifest::read_manifest(manifest).unwrap();
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut pool = StringPool::default();
        let Self { content, generator } = self;

        let definition = compile(&mut pool, content).unwrap();
        let global_group = definition.global();

        let group_name = global_group.name().resolve(&pool);
        generator.start_group(group_name);

        Self::generate_group(
            &pool,
            generator,
            &definition,
            definition.types_for_group(global_group),
            definition.messages_slice(global_group.messages()),
        );

        for group_symbol in definition.groups().iter().copied() {
            let group_name = group_symbol.name().resolve(&pool);
            generator.start_group(group_name);
            Self::generate_group(
                &pool,
                generator,
                &definition,
                definition.types_for_group(group_symbol),
                definition.messages_slice(group_symbol.messages()),
            );
            generator.end_group();
        }

        generator.end_group();

        Ok(())
    }

    fn generate_group(
        pool: &StringPool,
        generator: &mut G,
        definition: &ProtocolDefinition,
        types: &[Type],
        messages: &[MessageSymbolType],
    ) {
        for ty in types {
            match ty {
                Type::Numeric(_) | Type::Array(_) => {}
                Type::Struct(symbol) => {
                    let fields = definition.fields_slice(symbol.fields());
                    let symbol = EnrichedStructSymbol {
                        name: symbol.name().resolve(pool),
                        fields: FieldList::new(pool, definition.table(), fields, 0),
                    };
                    generator.generate_struct(symbol);
                }
                Type::Enum(symbol) => {
                    let symbol = EnrichedEnumSymbol {
                        name: symbol.name().resolve(pool),
                        repr: symbol.repr(),
                        variants: definition
                            .variants_slice(symbol.variants())
                            .iter()
                            .map(|v| ResolvedVariant {
                                name: v.ident().resolve(pool),
                                value: v.value(),
                            }),
                    };

                    generator.generate_enum(symbol);
                }
                Type::Flags(symbol) => {
                    let symbol = EnrichedFlagsSymbol {
                        name: symbol.name().resolve(pool),
                        options: OptionList::new(pool, definition.options_slice(symbol.values())),
                    };

                    generator.generate_flags(symbol);
                }
                Type::Bitset(symbol) => {
                    let symbol = EnrichedBitsetSymbol {
                        name: symbol.name().resolve(pool),
                    };

                    //self.generator.generate_bitset(symbol);
                }
            }
        }

        for message in messages {
            match message {
                MessageSymbolType::Single(symbol) => {
                    let symbol = EnrichedSingleMessageSymbol {
                        name: symbol.name().resolve(pool),
                        fields: std::iter::chain(
                            message_header(),
                            FieldList::new(
                                pool,
                                definition.table(),
                                definition.fields_slice(symbol.fields()),
                                HEADER_BYTES,
                            ),
                        ),
                    };

                    generator.generate_single_message(symbol);
                }
                MessageSymbolType::Stream(symbol) => {
                    let symbol = EnrichedStreamMessageSymbol {
                        name: symbol.name().resolve(pool),
                        timeout: symbol.timeout(),
                        kind: symbol.kind(),
                        fields: std::iter::chain(message_header(), std::iter::once(data_array())),
                        head: symbol.head().map(|fields| {
                            FieldList::new(
                                pool,
                                definition.table(),
                                definition.fields_slice(fields),
                                HEADER_BYTES,
                            )
                        }),
                        payload: FieldList::new(
                            pool,
                            definition.table(),
                            definition.fields_slice(symbol.payload()),
                            HEADER_BYTES,
                        ),
                        tail: symbol.tail().map(|fields| {
                            FieldList::new(
                                pool,
                                definition.table(),
                                definition.fields_slice(fields),
                                HEADER_BYTES,
                            )
                        }),
                    };
                    generator.generate_stream_message(symbol);
                }
            }
        }
    }
}

pub struct EnrichedSingleMessageSymbol<'s, I>
where
    I: Iterator<Item = EnrichedFieldSymbol<'s>>,
{
    pub name: &'s str,
    pub fields: I,
}

pub struct EnrichedStreamMessageSymbol<'s, F, I>
where
    F: Iterator<Item = EnrichedFieldSymbol<'s>>,
    I: Iterator<Item = EnrichedFieldSymbol<'s>>,
{
    pub name: &'s str,
    pub timeout: u8,
    pub kind: u8,

    pub fields: F,
    pub head: Option<I>,
    pub payload: I,
    pub tail: Option<I>,
}

fn message_header() -> [EnrichedFieldSymbol<'static>; 4] {
    [
        EnrichedFieldSymbol {
            name: "dst".into(),
            kind: EnrichedFieldType::Type("u8".into()),
        },
        EnrichedFieldSymbol {
            name: "group".into(),
            kind: EnrichedFieldType::Type("u8".into()),
        },
        EnrichedFieldSymbol {
            name: "src".into(),
            kind: EnrichedFieldType::Type("u8".into()),
        },
        EnrichedFieldSymbol {
            name: "kind".into(),
            kind: EnrichedFieldType::Type("u8".into()),
        },
    ]
}

fn data_array() -> EnrichedFieldSymbol<'static> {
    EnrichedFieldSymbol {
        name: "data".into(),
        kind: EnrichedFieldType::Array {
            kind: "u8".into(),
            len: MAX_PAYLOAD as u8,
        },
    }
}
