#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod proxy;

use std::{collections::HashMap, ffi::OsString, str::FromStr};

use creamy_manifest::Manifest;
use creamy_xmlc::{
    ProtocolDefinition, StringPoolResolver, compile,
    constraints::{HEADER_BYTES, MAX_PAYLOAD},
    model::symbols::{MessageSymbolType, StreamPayloadFieldSymbol, Type},
    utils::strpool::{StringId, StringPool},
};

use self::proxy::{
    BitsetValueList, EnrichedBitsetSymbol, EnrichedBitsetValueSymbol, EnrichedEnumSymbol,
    EnrichedFieldSymbol, EnrichedFieldType, EnrichedFlagsSymbol, EnrichedVariantSymbol, FieldList,
    FlagUnderlyingType, OptionList,
};

pub type GenResult = anyhow::Result<()>;

#[derive(Clone)]
pub struct EnrichedStructSymbol<'s, I>
where
    I: Iterator<Item = EnrichedFieldSymbol<'s>>,
{
    pub name: &'s str,
    pub fields: I,
}

pub trait CodeGenerator<'s> {
    fn start_group(&mut self, group: &'s str);

    fn generate_single_message<I>(&mut self, symbol: EnrichedSingleMessageSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedFieldSymbol<'s>>;

    fn generate_stream_message<F, P, I>(
        &mut self,
        symbol: EnrichedStreamMessageSymbol<'s, F, P, I>,
    ) where
        F: SymbolIterator<EnrichedFieldSymbol<'s>>,
        P: SymbolIterator<EnrichedFieldSymbol<'s>>,
        I: SymbolIterator<EnrichedFieldSymbol<'s>>;

    fn generate_bitset<I>(&mut self, symbol: EnrichedBitsetSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedBitsetValueSymbol<'s>>;

    fn generate_flags<I>(&mut self, symbol: EnrichedFlagsSymbol<'s, I>)
    where
        I: SymbolIterator<&'s str>;

    fn generate_enum<I>(&mut self, symbol: EnrichedEnumSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedVariantSymbol<'s>>;

    fn generate_struct<I>(&mut self, symbol: EnrichedStructSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedFieldSymbol<'s>>;

    fn end_group(&mut self);

    fn flush(&mut self) -> anyhow::Result<()>;
}

struct ProtocolContext {
    pool: StringPool,
    definition: ProtocolDefinition,
    dispatch_value_table: HashMap<String, HashMap<StringId, u32>>,
}

pub struct ProtocolLibrary {
    manifest: Manifest,
    inner: HashMap<String, ProtocolContext>,
}

impl ProtocolLibrary {
    #[must_use]
    pub fn new(manifest: &str) -> Self {
        let (_, manifest) = Manifest::read_manifest(manifest).unwrap();
        Self {
            manifest,
            inner: HashMap::new(),
        }
    }

    pub fn load_all(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(path).unwrap().filter_map(|result| {
            if let Ok(entry) = result
                && entry.path().extension() == Some(&OsString::from_str("xml").unwrap())
            {
                return Some(entry);
            }

            None
        }) {
            let content = std::fs::read_to_string(entry.path())?;
            let mut pool = StringPool::default();
            let definition = compile(&mut pool, &content).unwrap();
            self.inner.insert(
                definition.name().resolve(&pool).to_string(),
                ProtocolContext {
                    //dispatch_value_table: Self::create_dispatch_value_table(&pool, &definition),
                    pool,
                    definition,
                    dispatch_value_table: HashMap::new(),
                },
            );
        }

        Ok(())
    }

    //fn create_dispatch_value_table(
    //    pool: &StringPool,
    //    definition: &ProtocolDefinition,
    //) -> HashMap<String, HashMap<MessageSymbolType, u32>> {
    //
    //}

    fn create_dispatch_values(&mut self) {
        for protocol in self.manifest.requested_protocols() {
            if let Some(id) = protocol.static_group_id() {
                let id = id.get();
                let mut split = protocol.name().split_terminator('.');
                let protocol_name = split.next().unwrap();
                let group_name = split.next().unwrap();

                let ProtocolContext {
                    pool,
                    definition,
                    dispatch_value_table,
                } = self.inner.get_mut(protocol_name).unwrap();

                let group_name_id = pool.get_id(group_name);

                let group = definition
                    .groups()
                    .iter()
                    .find(|g| g.name() == group_name_id)
                    .unwrap();

                let dispatch_values = dispatch_value_table
                    .entry(group_name.to_string())
                    .or_default();

                for message_symbol in definition.messages_slice(group.messages()) {
                    // Valid form of the dispatch value: [0x_00_FF_00_FF]
                    let dispatch_value = (id as u32) << 16 | message_symbol.kind() as u32;
                    dispatch_values.insert(message_symbol.name(), dispatch_value);
                }
            }
        }
    }
}

pub struct Codegen {
    library: ProtocolLibrary,
}

impl Codegen {
    #[must_use]
    pub fn new(mut library: ProtocolLibrary) -> Self {
        library.create_dispatch_values();
        Self { library }
    }

    pub fn run<'s, G: CodeGenerator<'s>>(
        &'s mut self,
        protocol_name: &str,
        generator: &mut G,
    ) -> anyhow::Result<()> {
        let ProtocolContext {
            pool,
            definition,
            dispatch_value_table,
        } = self.library.inner.get(protocol_name).unwrap();

        let global_group = definition.global();

        let group_name = global_group.name().resolve(pool);
        generator.start_group(group_name);

        Self::generate_group(
            pool,
            definition,
            &HashMap::new(),
            //dispatch_value_table.get(group_name).unwrap(),
            definition.types_for_group(global_group),
            definition.messages_slice(global_group.messages()),
            generator,
        );

        for group_symbol in definition.groups().iter().copied() {
            let group_name = group_symbol.name().resolve(pool);
            generator.start_group(group_name);
            Self::generate_group(
                pool,
                definition,
                dispatch_value_table.get(group_name).unwrap(),
                definition.types_for_group(group_symbol),
                definition.messages_slice(group_symbol.messages()),
                generator,
            );
            generator.end_group();
        }

        generator.end_group();
        generator.flush()
    }

    #[allow(clippy::too_many_lines)]
    fn generate_group<'s, G: CodeGenerator<'s>>(
        pool: &'s StringPool,
        definition: &'s ProtocolDefinition,
        dispatch_value_table: &HashMap<StringId, u32>,
        types: &[Type],
        messages: &[MessageSymbolType],
        generator: &mut G,
    ) {
        for ty in types {
            match ty {
                Type::Numeric(_) | Type::Array(_) => {}
                Type::Struct(symbol) => {
                    let fields = definition.fields_slice(symbol.fields());
                    let symbol = EnrichedStructSymbol {
                        name: symbol.name().resolve(pool),
                        fields: FieldList::new(pool, definition.table(), fields.iter().copied(), 0),
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
                            .map(|v| EnrichedVariantSymbol {
                                name: v.ident().resolve(pool),
                                value: v.value(),
                            }),
                    };

                    generator.generate_enum(symbol);
                }
                Type::Flags(symbol) => {
                    let symbol = EnrichedFlagsSymbol {
                        name: symbol.name().resolve(pool),
                        underlying_type: match symbol.values().len() {
                            1..=8 => FlagUnderlyingType::U8,
                            9..=16 => FlagUnderlyingType::U16,
                            17..=32 => FlagUnderlyingType::U32,
                            33..=64 => FlagUnderlyingType::U64,
                            65..=128 => FlagUnderlyingType::U128,
                            other => unreachable!("Unreachable length: {other}"),
                        },
                        options: OptionList::new(pool, definition.options_slice(symbol.values())),
                    };

                    generator.generate_flags(symbol);
                }
                Type::Bitset(symbol) => {
                    let slice = definition.bvalues_slice(symbol.values());
                    let symbol = EnrichedBitsetSymbol {
                        name: symbol.name().resolve(pool),
                        values: BitsetValueList::new(pool, slice),
                    };

                    generator.generate_bitset(symbol);
                }
            }
        }

        for message in messages {
            match message {
                MessageSymbolType::Single(symbol) => {
                    let symbol = EnrichedSingleMessageSymbol {
                        name: symbol.name().resolve(pool),
                        kind: symbol.kind(),
                        dispatch_value: dispatch_value_table.get(&symbol.name()).copied(),
                        fields: std::iter::chain(
                            message_header(),
                            FieldList::new(
                                pool,
                                definition.table(),
                                definition.fields_slice(symbol.fields()).iter().copied(),
                                HEADER_BYTES,
                            ),
                        ),
                    };

                    generator.generate_single_message(symbol);
                }
                MessageSymbolType::Stream(symbol) => {
                    let payload_slice = definition
                        .payload_slice(symbol.payload())
                        .iter()
                        .map(|p| match p {
                            StreamPayloadFieldSymbol::Field(symbol) => *symbol,
                            StreamPayloadFieldSymbol::Array(_) => todo!("enrich array"),
                        })
                        .collect::<Vec<_>>();

                    let symbol = EnrichedStreamMessageSymbol {
                        name: symbol.name().resolve(pool),
                        timeout: symbol.timeout(),
                        kind: symbol.kind(),
                        dispatch_value: dispatch_value_table.get(&symbol.name()).copied(),
                        fields: std::iter::chain(message_header(), std::iter::once(data_array())),
                        head: symbol.head().map(|fields| {
                            FieldList::new(
                                pool,
                                definition.table(),
                                definition.fields_slice(fields).iter().copied(),
                                HEADER_BYTES,
                            )
                        }),
                        payload: FieldList::new(
                            pool,
                            definition.table(),
                            //TODO: allow arrays
                            payload_slice.into_iter(),
                            HEADER_BYTES,
                        ),
                        tail: symbol.tail().map(|fields| {
                            FieldList::new(
                                pool,
                                definition.table(),
                                definition.fields_slice(fields).iter().copied(),
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
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    pub name: &'s str,
    pub kind: u8,
    pub dispatch_value: Option<u32>,
    pub fields: I,
}

pub struct EnrichedStreamMessageSymbol<'s, F, P, I>
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    P: SymbolIterator<EnrichedFieldSymbol<'s>>,
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    pub name: &'s str,
    pub kind: u8,
    pub dispatch_value: Option<u32>,
    pub timeout: u8,

    pub fields: F,
    pub head: Option<I>,
    pub payload: P,
    pub tail: Option<I>,
}

fn message_header() -> [EnrichedFieldSymbol<'static>; 4] {
    [
        EnrichedFieldSymbol {
            name: "dst".into(),
            kind: EnrichedFieldType::Type("u8".into()),
            is_padding: false,
        },
        EnrichedFieldSymbol {
            name: "group".into(),
            kind: EnrichedFieldType::Type("u8".into()),
            is_padding: false,
        },
        EnrichedFieldSymbol {
            name: "src".into(),
            kind: EnrichedFieldType::Type("u8".into()),
            is_padding: false,
        },
        EnrichedFieldSymbol {
            name: "kind".into(),
            kind: EnrichedFieldType::Type("u8".into()),
            is_padding: false,
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
        is_padding: false,
    }
}

pub trait SymbolIterator<S>: Iterator<Item = S> + Clone {}

impl<I, F, S> SymbolIterator<S> for std::iter::Map<I, F>
where
    I: Iterator + Clone,
    F: FnMut(I::Item) -> S + Clone,
{
}

impl<A, B, S> SymbolIterator<S> for std::iter::Chain<A, B>
where
    A: Iterator<Item = S> + Clone,
    B: Iterator<Item = S> + Clone,
{
}
