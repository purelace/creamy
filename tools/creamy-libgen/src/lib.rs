//#![deny(clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]

mod api;

pub mod manifest {
    pub use creamy_manifest::*;
}

pub mod prelude {
    pub use creamy_phf::PerfectHashTable;
    pub use creamy_xmlc::{
        Api,
        symbols::{PrimitiveRepr, SpecialType},
    };

    pub use crate::{
        CodeGenerator, Codegen, GenResult, Path, ProtocolLibrary, SymbolIterator, Target,
        api::{LinkSectionEntry, LinkSections, ManifestData, ProtocolEntry},
        proxy::{
            EnrichedBitsetSymbol, EnrichedBitsetValueSymbol, EnrichedEnumSymbol,
            EnrichedFieldSymbol, EnrichedFieldType, EnrichedFlagsSymbol, EnrichedStructSymbol,
            EnrichedVariantSymbol, FlagUnderlyingType,
        },
    };
}

pub mod proxy;
mod utils;
use std::{collections::HashMap, marker::PhantomData};

use creamy_manifest::Manifest;
use creamy_phf::{PerfectHashTable, generate_perfect_hash_table};
use creamy_xmlc::{
    SemanticModel,
    symbols::{LayoutEntry, MessageSymbolType, Type},
    utils::{
        Direction,
        strpool::{StringId, StringPoolResolver},
    },
};
pub use utils::{AbsolutePath, Path};

use self::{
    api::LinkSections,
    proxy::{
        BitsetValueList, EnrichedBitsetSymbol, EnrichedBitsetValueSymbol, EnrichedEnumSymbol,
        EnrichedFieldSymbol, EnrichedFlagsSymbol, EnrichedLayoutEntry, EnrichedMessageSymbol,
        EnrichedObjectSymbol, EnrichedSerializationLayout, EnrichedStructSymbol,
        EnrichedVariantSymbol, FieldList, FlagUnderlyingType, MessageType, OptionList,
    },
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

pub trait CodeGenerator<'s> {
    //TODO: start/end namespace
    fn start_protocol(&mut self, protocol: &'s str, id: Option<u8>);

    fn generate_message<F, C>(&mut self, symbol: EnrichedMessageSymbol<'s, F, C>)
    where
        F: SymbolIterator<EnrichedFieldSymbol<'s>>,
        C: SymbolIterator<EnrichedFieldSymbol<'s>>;

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

    fn generate_object<I>(&mut self, symbol: EnrichedObjectSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedFieldSymbol<'s>>;

    fn end_protocol(&mut self);

    fn generate_dispatcher<P, M>(&mut self, protocols: P, messages: M)
    where
        P: Iterator<Item = Path>,
        M: Iterator<Item = Path>;

    fn generate_plugin_protocol_table(&mut self, pht: PerfectHashTable);

    fn generate_host_protocol_table(&mut self, pht: PerfectHashTable);

    fn generate_link_section(&mut self, data: LinkSections);

    //fn generate_host_package(&mut self, binary: Vec<u8>);

    fn flush(&mut self) -> anyhow::Result<()>;
}

pub struct ProtocolLibrary {
    manifest: Manifest,
    inner: HashMap<String, SemanticModel>,
}

impl ProtocolLibrary {
    /// Creates a new [`ProtocolLibrary`].
    ///
    /// # Panics
    ///
    /// Panics if manifest deserialization is failed
    #[must_use]
    pub fn new(manifest: &str) -> Self {
        Self {
            manifest: Manifest::read_manifest(manifest).expect("Cannot read manifest"),
            inner: HashMap::new(),
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
            if e.path().extension().is_some_and(|e| e == "cmp") {
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
        let path = path.as_ref();
        for entry in std::fs::read_dir(path)?.filter_map(Self::filter) {
            let entry = entry?;
            let content = std::fs::read_to_string(entry.path())?;
            let model = match SemanticModel::compile(&content) {
                Ok(v) => v,
                Err(d) => {
                    d.print(&path.as_os_str().to_string_lossy(), &content);
                    panic!();
                }
            };
            self.inner.insert(model.name().to_string(), model);
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

        let ProtocolLibrary { manifest, inner } = &mut self.library;

        generator.start_protocol("generated", None);

        Self::generate_protocols(
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
                Self::generate_link_section(manifest, inner, generator);
            }
            Target::Host => {
                Self::generate_host_protocol_table(manifest, generator);
                //Self::generate_link_section(manifest, inner, generator);
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
        manifest: &'s Manifest,
        library: &'s HashMap<String, SemanticModel>,
        target: Target,
        message_paths: &mut Vec<Path>,
        protocol_paths: &mut Vec<Path>,
        generator: &mut G,
    ) {
        let mut protocol_id = u8::from(!matches!(target, Target::Sdk | Target::Engine)) + 1;

        for (name, protocol) in manifest.requested_protocols() {
            let model = library.get(name.as_str()).unwrap();

            generator.start_protocol(name, Some(protocol_id));
            protocol_paths.push(Path::from_absolute(AbsolutePath::from_iter([
                name.as_str(),
                "ID",
            ])));

            let mut dispatch_values: HashMap<StringId, (u8, u32)> = HashMap::new();
            for message_symbol in model
                .stable()
                .iter()
                .chain(model.extension())
                .chain(model.experimental())
            {
                // Valid form of the dispatch value: [0x_00_FF_00_FF]
                let dispatch_value = u32::from(protocol_id) << 16 | u32::from(message_symbol.kind);
                dispatch_values.insert(message_symbol.ident, (protocol_id, dispatch_value));

                let dir = message_symbol.direction;

                let should_add = dir == Direction::Duplex
                    || (protocol.provide() && dir == Direction::Incoming)
                    || (!protocol.provide() && dir == Direction::Outgoing);

                if should_add {
                    message_paths.push(Path::from_absolute(AbsolutePath::from_iter([
                        name.as_str(),
                        message_symbol.ident.resolve(model.pool()),
                    ])));
                }
            }

            Self::generate_protocol(
                model,
                &dispatch_values,
                &AbsolutePath::from_iter([name.as_str()]),
                generator,
            );

            generator.end_protocol();

            protocol_id += 1;
        }
    }

    fn generate_link_section_<'s, G: CodeGenerator<'s>>(
        manifest: &'s Manifest,
        models: &HashMap<String, SemanticModel>,
        generator: &mut G,
    ) {
        let data = LinkSections::new(manifest, models);
        generator.generate_link_section(data);

        //let binary =
        //    creamy_plugin_package::compilation::compile_to_binary(creamy_dir, Box::default())
        //        .unwrap();
        //let mut vec = vec![];
        //binary.write_ne(&mut vec).unwrap();

        //generator.generate_host_package(vec);
    }

    fn generate_host_protocol_table<'s, G: CodeGenerator<'s>>(
        manifest: &Manifest,
        generator: &mut G,
    ) {
        generator.generate_host_protocol_table(Self::generate_hash_table(manifest));
    }

    fn generate_link_section<'s, G: CodeGenerator<'s>>(
        manifest: &Manifest,
        models: &HashMap<String, SemanticModel>,
        generator: &mut G,
    ) {
        let data = LinkSections::new(manifest, models);
        generator.generate_link_section(data);
        //generator.generate_plugin_protocol_table(Self::generate_hash_table(manifest));
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
        model: &'s SemanticModel,
        module_path: &AbsolutePath,
        generator: &mut G,
    ) {
        for ty in model.types() {
            match ty {
                Type::Numeric(_) => {}
                Type::Enum(s) => {
                    let symbol = EnrichedEnumSymbol {
                        name: s.ident.resolve(model.pool()),
                        repr: s.repr,
                        variants: s.variants.iter().map(|v| EnrichedVariantSymbol {
                            name: v.ident.resolve(model.pool()),
                            value: v.value,
                        }),
                    };
                    generator.generate_enum(symbol);
                }
                Type::Flags(s) => {
                    let symbol = EnrichedFlagsSymbol {
                        name: s.ident.resolve(model.pool()),
                        underlying_type: match s.options.len() {
                            1..=8 => FlagUnderlyingType::U8,
                            9..=16 => FlagUnderlyingType::U16,
                            17..=32 => FlagUnderlyingType::U32,
                            33..=64 => FlagUnderlyingType::U64,
                            65..=128 => FlagUnderlyingType::U128,
                            other => unreachable!("Unreachable length: {other}"),
                        },
                        options: OptionList::new(model.pool(), &s.options),
                    };
                    generator.generate_flags(symbol);
                }
                Type::Bitset(s) => {
                    let symbol = EnrichedBitsetSymbol {
                        name: s.ident.resolve(model.pool()),
                        values: BitsetValueList::new(model.pool(), &s.values),
                    };
                    generator.generate_bitset(symbol);
                }
                Type::Struct(s) => {
                    let symbol = EnrichedStructSymbol {
                        name: s.ident.resolve(model.pool()),
                        fields: FieldList::new(
                            module_path.clone(),
                            model.pool(),
                            model.table(),
                            s.fields.iter().copied(),
                        ),
                    };
                    generator.generate_struct(symbol);
                }
                Type::Object(s) => {
                    let symbol = EnrichedObjectSymbol {
                        name: s.ident.resolve(model.pool()),
                        fields: FieldList::new(
                            module_path.clone(),
                            model.pool(),
                            model.table(),
                            s.fields.iter().copied(),
                        ),
                    };
                    generator.generate_object(symbol);
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn generate_protocol<'s, G: CodeGenerator<'s>>(
        model: &'s SemanticModel,
        dispatch_value_table: &HashMap<StringId, (u8, u32)>, //protocol, value
        module_path: &AbsolutePath,
        generator: &mut G,
    ) {
        Self::generate_types(model, module_path, generator);

        for symbol in model
            .stable()
            .iter()
            .chain(model.extension())
            .chain(model.experimental())
        {
            let (protocol, dispatch_value) =
                dispatch_value_table.get(&symbol.ident).copied().unwrap();

            let message_type = match &symbol.message_type {
                MessageSymbolType::Single => MessageType::Single,
                MessageSymbolType::Stream {
                    serialization_layout,
                    lightweight_content,
                } => {
                    let items = serialization_layout
                        .items()
                        .iter()
                        .map(|e| match e {
                            LayoutEntry::ArrayLength { name } => EnrichedLayoutEntry::ArrayLength {
                                name: name.resolve(model.pool()),
                            },
                            LayoutEntry::ArrayWrite { name } => EnrichedLayoutEntry::ArrayWrite {
                                name: name.resolve(model.pool()),
                            },
                            &LayoutEntry::Slice { to, kind } => EnrichedLayoutEntry::Slice {
                                to,
                                kind: model.table().get_type(kind).ident().resolve(model.pool()),
                            },
                            &LayoutEntry::LastSlice { to } => EnrichedLayoutEntry::LastSlice { to },
                        })
                        .collect::<Vec<_>>();

                    MessageType::Stream {
                        _phantom: PhantomData,
                        serialization_layout: EnrichedSerializationLayout { items },
                        content_fields: FieldList::new(
                            module_path.clone(),
                            model.pool(),
                            model.table(),
                            lightweight_content.iter().copied(),
                        ),
                    }
                }
            };

            let symbol = EnrichedMessageSymbol {
                name: symbol.ident.resolve(model.pool()),
                kind: symbol.kind,
                protocol,
                dispatch_value,
                fields: FieldList::new(
                    module_path.clone(),
                    model.pool(),
                    model.table(),
                    symbol.fields.iter().copied(),
                ),
                message_type,
            };

            generator.generate_message(symbol);
        }
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
