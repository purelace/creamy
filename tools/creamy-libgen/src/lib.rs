//#![deny(clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]

pub mod package {
    pub use creamy_plugin_package::*;
}

pub mod prelude {
    pub use creamy_phf::PerfectHashTable;
    pub use creamy_plugin_package::model::symbols::{PrimitiveRepr, VariantValue};

    pub use crate::{
        CodeGenerator, Codegen, EnrichedSingleMessageSymbol, EnrichedStreamMessageSymbol,
        EnrichedStructSymbol, GenResult, Path, ProtocolLibrary, SymbolIterator, Target,
        proxy::{
            EnrichedBitsetSymbol, EnrichedBitsetValueSymbol, EnrichedEnumSymbol,
            EnrichedFieldSymbol, EnrichedFieldType, EnrichedFlagsSymbol, EnrichedStreamFieldSymbol,
            EnrichedVariantSymbol, FlagUnderlyingType,
        },
    };
}

pub mod proxy;
mod utils;
use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use creamy_phf::{PerfectHashTable, generate_perfect_hash_table};
use creamy_plugin_package::manifest::Manifest;
use creamy_xmlc::{
    compile,
    model::{
        Direction,
        constraints::MAX_PAYLOAD,
        definition::{BinaryWriter, ProtocolModel},
        strpool::{StringId, StringPool, StringPoolResolver},
        symbols::{MessageSymbolType, Type},
    },
};
pub use utils::{AbsolutePath, Path};

use self::proxy::{
    BitsetValueList, EnrichedBitsetSymbol, EnrichedBitsetValueSymbol, EnrichedEnumSymbol,
    EnrichedFieldSymbol, EnrichedFieldType, EnrichedFlagsSymbol, EnrichedStreamFieldSymbol,
    EnrichedVariantSymbol, FieldList, FlagUnderlyingType, OptionList, SpecialFieldType,
    StreamFieldList,
};

#[derive(Clone, Copy)]
pub enum Target {
    #[doc(hidden)]
    Engine,
    Plugin,
    Host,
    #[doc(hidden)]
    Sdk,
}

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
    //TODO: start/end namespace
    fn start_protocol(&mut self, protocol: &'s str, id: Option<u8>);

    fn generate_single_message<I>(&mut self, symbol: EnrichedSingleMessageSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedFieldSymbol<'s>>;

    fn generate_stream_message<F, C>(&mut self, symbol: EnrichedStreamMessageSymbol<'s, F, C>)
    where
        F: SymbolIterator<EnrichedFieldSymbol<'s>>,
        C: SymbolIterator<EnrichedStreamFieldSymbol<'s>>;

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

    fn end_protocol(&mut self);

    fn generate_dispatcher<P, M>(&mut self, protocols: P, messages: M)
    where
        P: Iterator<Item = Path>,
        M: Iterator<Item = Path>;

    fn generate_plugin_protocol_table(&mut self, pht: PerfectHashTable);

    fn generate_host_protocol_table(&mut self, pht: PerfectHashTable);

    fn generate_host_package(&mut self, binary: Vec<u8>);

    fn flush(&mut self) -> anyhow::Result<()>;
}

pub struct ProtocolLibrary {
    pool: StringPool,
    manifest: Manifest,
    inner: HashMap<String, ProtocolModel>,
    creamy_dir: PathBuf,
}

impl ProtocolLibrary {
    /// Creates a new [`ProtocolLibrary`].
    ///
    /// # Panics
    ///
    /// Panics if manifest deserialization is failed
    #[must_use]
    pub fn new(manifest: &str, creamy_dir: PathBuf) -> Self {
        Self {
            pool: StringPool::default(),
            manifest: Manifest::read_manifest(manifest).expect("Cannot read manifest"),
            inner: HashMap::new(),
            creamy_dir,
        }
    }

    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn filter(
        result: Result<std::fs::DirEntry, std::io::Error>,
    ) -> Option<Result<std::fs::DirEntry, std::io::Error>> {
        let result = result.map(|e| {
            if e.path().extension().is_some_and(|e| e == "xml") {
                Some(e)
            } else {
                None
            }
        });
        result.transpose()
    }

    /// Reads all xml files from the given path and compiles them
    ///
    /// # Panics
    ///
    /// Panics if compilation is failed
    ///
    /// # Errors
    ///
    /// This function will return an error if path is not exist.
    pub fn load_all(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(path)?.filter_map(Self::filter) {
            let entry = entry?;
            let content = std::fs::read_to_string(entry.path())?;
            let definition = match compile(&mut self.pool, &content) {
                Ok(v) => v,
                Err(d) => {
                    d.print(&content);
                    panic!();
                }
            };
            self.inner.insert(
                definition.name().resolve(&self.pool).to_string(),
                definition,
            );
        }

        Ok(())
    }
}

pub struct Codegen {
    library: ProtocolLibrary,
    target: Target,
}

impl Codegen {
    #[must_use]
    pub const fn new(library: ProtocolLibrary, target: Target) -> Self {
        Self { library, target }
    }

    #[must_use]
    pub const fn library(&self) -> &ProtocolLibrary {
        &self.library
    }

    pub fn run<'s, G: CodeGenerator<'s>>(&'s mut self, generator: &mut G) -> anyhow::Result<()> {
        let mut message_paths = vec![];
        let mut protocol_paths = vec![];

        let ProtocolLibrary {
            pool,
            manifest,
            inner,
            creamy_dir,
        } = &mut self.library;

        generator.start_protocol("generated", None);

        Self::generate_protocols(
            pool,
            manifest,
            inner,
            self.target,
            &mut message_paths,
            &mut protocol_paths,
            generator,
        );

        generator.start_protocol("dispatcher", None);

        generator.generate_dispatcher(protocol_paths.into_iter(), message_paths.into_iter());

        generator.end_protocol();

        generator.start_protocol("metadata", None);

        match self.target {
            Target::Plugin => {
                Self::generate_plugin_protocol_table(manifest, generator);
            }
            Target::Host => {
                Self::generate_host_protocol_table(manifest, generator);
                Self::generate_host_package(creamy_dir.clone(), generator);
            }
            Target::Engine | Target::Sdk => {}
        }

        generator.end_protocol();

        generator.end_protocol();
        generator.flush()?;

        Ok(())
    }

    #[allow(clippy::explicit_counter_loop)]
    fn generate_protocols<'s, G: CodeGenerator<'s>>(
        pool: &'s StringPool,
        manifest: &'s Manifest,
        library: &'s mut HashMap<String, ProtocolModel>,
        target: Target,
        message_paths: &mut Vec<Path>,
        protocol_paths: &mut Vec<Path>,
        generator: &mut G,
    ) {
        let mut protocol_id = u8::from(!matches!(target, Target::Sdk | Target::Engine)) + 1;

        for (name, protocol) in manifest.requested_protocols() {
            let model = library.get(name.as_str()).unwrap();

            generator.start_protocol(model.name().resolve(pool), Some(protocol_id));
            protocol_paths.push(Path::from_absolute(AbsolutePath::from_iter([
                name.as_str(),
                "ID",
            ])));

            let mut dispatch_values: HashMap<StringId, (u8, u32)> = HashMap::new();
            for message_symbol in model.messages() {
                // Valid form of the dispatch value: [0x_00_FF_00_FF]
                let dispatch_value =
                    u32::from(protocol_id) << 16 | u32::from(message_symbol.kind());
                dispatch_values.insert(message_symbol.ident(), (protocol_id, dispatch_value));

                let dir = message_symbol.direction();

                // Условие включения:
                // 1. Это Duplex (нужен всегда)
                // 2. Или группа поставляется И направление Outgoing
                // 3. Или группа НЕ поставляется И направление Incoming
                let should_add = dir == Direction::Duplex
                    || (protocol.provide() && dir == Direction::Outgoing)
                    || (!protocol.provide() && dir == Direction::Incoming);

                if should_add {
                    message_paths.push(Path::from_absolute(AbsolutePath::from_iter([
                        name.as_str(),
                        message_symbol.ident().resolve(pool),
                    ])));
                }
            }

            Self::generate_protocol(
                pool,
                model,
                &dispatch_values,
                &AbsolutePath::from_iter([name.as_str()]),
                model.types(),
                model.messages(),
                generator,
            );

            generator.end_protocol();

            protocol_id += 1;
        }
    }

    fn generate_host_package<'s, G: CodeGenerator<'s>>(creamy_dir: PathBuf, generator: &mut G) {
        let binary =
            creamy_plugin_package::compilation::compile_to_binary(creamy_dir, Box::default())
                .unwrap();
        let mut vec = vec![];
        binary.write_ne(&mut vec).unwrap();

        generator.generate_host_package(vec);
    }

    fn generate_host_protocol_table<'s, G: CodeGenerator<'s>>(
        manifest: &Manifest,
        generator: &mut G,
    ) {
        generator.generate_host_protocol_table(Self::generate_hash_table(manifest));
    }

    fn generate_plugin_protocol_table<'s, G: CodeGenerator<'s>>(
        manifest: &Manifest,
        generator: &mut G,
    ) {
        generator.generate_plugin_protocol_table(Self::generate_hash_table(manifest));
    }

    fn generate_hash_table(manifest: &Manifest) -> PerfectHashTable {
        let samples = manifest
            .requested_protocols()
            .keys()
            .cloned()
            .enumerate()
            .map(|(index, path)| (path, index as u8 + 2))
            .collect::<Vec<_>>();

        let size_of_table: usize = samples.len();
        assert!(u8::try_from(size_of_table).is_ok());
        let size_of_table = size_of_table as u8;

        generate_perfect_hash_table(
            size_of_table.div_ceil(4),
            size_of_table,
            samples.iter().map(|(a, b)| (a.as_str(), *b)),
        )
    }

    fn generate_types<'s, G: CodeGenerator<'s>>(
        pool: &'s StringPool,
        model: &'s ProtocolModel,
        module_path: &AbsolutePath,
        types: &[Type],
        generator: &mut G,
    ) {
        for ty in types {
            match ty {
                Type::Numeric(_) | Type::Array(_) => {}
                Type::Struct(symbol) => {
                    let fields = model.fields_range(symbol.fields());
                    let symbol = EnrichedStructSymbol {
                        name: symbol.ident().resolve(pool),
                        fields: FieldList::new(
                            module_path.clone(),
                            pool,
                            model.table(),
                            fields.iter().copied(),
                        ),
                    };
                    generator.generate_struct(symbol);
                }
                Type::Enum(symbol) => {
                    let symbol = EnrichedEnumSymbol {
                        name: symbol.ident().resolve(pool),
                        repr: symbol.repr(),
                        variants: model.variants_slice(symbol.variants()).iter().map(|v| {
                            EnrichedVariantSymbol {
                                name: v.ident().resolve(pool),
                                value: v.value(),
                            }
                        }),
                    };

                    generator.generate_enum(symbol);
                }
                Type::Flags(symbol) => {
                    let symbol = EnrichedFlagsSymbol {
                        name: symbol.ident().resolve(pool),
                        underlying_type: match symbol.values().len() {
                            1..=8 => FlagUnderlyingType::U8,
                            9..=16 => FlagUnderlyingType::U16,
                            17..=32 => FlagUnderlyingType::U32,
                            33..=64 => FlagUnderlyingType::U64,
                            65..=128 => FlagUnderlyingType::U128,
                            other => unreachable!("Unreachable length: {other}"),
                        },
                        options: OptionList::new(pool, model.options_slice(symbol.values())),
                    };

                    generator.generate_flags(symbol);
                }
                Type::Bitset(symbol) => {
                    let slice = model.bvalues_slice(symbol.values());
                    let symbol = EnrichedBitsetSymbol {
                        name: symbol.ident().resolve(pool),
                        values: BitsetValueList::new(pool, slice),
                    };

                    generator.generate_bitset(symbol);
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn generate_protocol<'s, G: CodeGenerator<'s>>(
        pool: &'s StringPool,
        model: &'s ProtocolModel,
        dispatch_value_table: &HashMap<StringId, (u8, u32)>, //protocol, value
        module_path: &AbsolutePath,
        types: &[Type],
        messages: &[MessageSymbolType],
        generator: &mut G,
    ) {
        Self::generate_types(pool, model, module_path, types, generator);

        for message in messages {
            let (protocol, dispatch_value) =
                dispatch_value_table.get(&message.ident()).copied().unwrap();

            match message {
                MessageSymbolType::Single(symbol) => {
                    let symbol = EnrichedSingleMessageSymbol {
                        name: symbol.ident().resolve(pool),
                        kind: symbol.kind(),
                        protocol,
                        dispatch_value,
                        fields: std::iter::chain(
                            message_header(),
                            FieldList::new(
                                module_path.clone(),
                                pool,
                                model.table(),
                                model.fields_range(symbol.fields()).iter().copied(),
                            ),
                        ),
                    };

                    generator.generate_single_message(symbol);
                }
                MessageSymbolType::Stream(symbol) => {
                    let symbol = EnrichedStreamMessageSymbol {
                        name: symbol.ident().resolve(pool),
                        timeout: symbol.timeout(),

                        protocol,
                        kind: symbol.kind(),
                        dispatch_value,

                        message_fields: std::iter::chain(message_header(), data_array()),
                        content_fields: StreamFieldList::new(
                            module_path.clone(),
                            pool,
                            model.table(),
                            model.stream_fields_range(symbol.content()).iter().copied(),
                        ),
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
    /* Constant Values */
    pub protocol: u8,
    pub kind: u8,
    pub dispatch_value: u32,

    pub fields: I,
}

pub struct EnrichedStreamMessageSymbol<'s, F, C>
where
    F: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
{
    pub name: &'s str,

    /* Constant values */
    pub kind: u8,
    pub protocol: u8,
    pub dispatch_value: u32,

    pub timeout: u8,

    pub message_fields: F,
    pub content_fields: C,
}

fn message_header() -> [EnrichedFieldSymbol<'static>; 4] {
    [
        EnrichedFieldSymbol {
            name: Cow::Borrowed("dst"),
            kind: EnrichedFieldType::Special(SpecialFieldType::SubscriberId),
            is_padding: false,
        },
        EnrichedFieldSymbol {
            name: Cow::Borrowed("protocol"),
            kind: EnrichedFieldType::Special(SpecialFieldType::ProtocolId),
            is_padding: false,
        },
        EnrichedFieldSymbol {
            name: Cow::Borrowed("src"),
            kind: EnrichedFieldType::Special(SpecialFieldType::SubscriberId),
            is_padding: false,
        },
        EnrichedFieldSymbol {
            name: Cow::Borrowed("kind"),
            kind: EnrichedFieldType::Type {
                name: Path::from_global("u8"),
            },
            is_padding: false,
        },
    ]
}

fn data_array() -> [EnrichedFieldSymbol<'static>; 2] {
    [
        EnrichedFieldSymbol {
            name: Cow::Borrowed("meta"),
            kind: EnrichedFieldType::Type {
                name: Path::from_global("u8"),
            },
            is_padding: false,
        },
        EnrichedFieldSymbol {
            name: Cow::Borrowed("data"),
            kind: EnrichedFieldType::Array {
                kind: Cow::Borrowed("u8"),
                #[allow(clippy::cast_possible_truncation)]
                len: MAX_PAYLOAD as u8 - 1,
            },
            is_padding: false,
        },
    ]
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
