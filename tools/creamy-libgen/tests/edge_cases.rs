use std::{fs::File, io::Write};

use creamy_libgen::{
    CodeGenerator, Codegen, ProtocolLibrary, Target,
    proxy::{EnrichedFieldSymbol, EnrichedFieldType},
};
use tempfile::TempDir;

const MANIFEST: &str = r#"
[package]
id = "org.creamy.system"
name = "system"
version = "1.0.0"
description = "Builtin package"
repository = "https://github.com/purelace/creamy"
authors = [ "selrisu <myirisuchan@gmail.com>" ]

[protocols]
system = { version = "1.0.0", groups=["builtin"], provide = true  }
"#;

fn setup_root() -> std::io::Result<TempDir> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join(".creamy");
    std::fs::create_dir_all(path.join("definitions"))?;
    File::create_new(path.join("manifest.toml"))
        .and_then(|mut f| f.write_all(MANIFEST.as_bytes()))?;

    Ok(dir)
}

struct Mock;
impl<'s> CodeGenerator<'s> for Mock {
    fn start_group(&mut self, _: &'s str, _: Option<u8>) {}

    fn generate_single_message<I>(&mut self, _: creamy_libgen::EnrichedSingleMessageSymbol<'s, I>)
    where
        I: creamy_libgen::SymbolIterator<creamy_libgen::proxy::EnrichedFieldSymbol<'s>>,
    {
    }

    fn generate_stream_message<F, C>(
        &mut self,
        _: creamy_libgen::EnrichedStreamMessageSymbol<'s, F, C>,
    ) where
        F: creamy_libgen::SymbolIterator<creamy_libgen::proxy::EnrichedFieldSymbol<'s>>,
        C: creamy_libgen::SymbolIterator<creamy_libgen::proxy::EnrichedStreamFieldSymbol<'s>>,
    {
    }

    fn generate_bitset<I>(&mut self, _: creamy_libgen::proxy::EnrichedBitsetSymbol<'s, I>)
    where
        I: creamy_libgen::SymbolIterator<creamy_libgen::proxy::EnrichedBitsetValueSymbol<'s>>,
    {
        todo!()
    }

    fn generate_flags<I>(&mut self, _: creamy_libgen::proxy::EnrichedFlagsSymbol<'s, I>)
    where
        I: creamy_libgen::SymbolIterator<&'s str>,
    {
        todo!()
    }

    fn generate_enum<I>(&mut self, _: creamy_libgen::proxy::EnrichedEnumSymbol<'s, I>)
    where
        I: creamy_libgen::SymbolIterator<creamy_libgen::proxy::EnrichedVariantSymbol<'s>>,
    {
    }

    fn generate_struct<I>(&mut self, symbol: creamy_libgen::EnrichedStructSymbol<'s, I>)
    where
        I: creamy_libgen::SymbolIterator<creamy_libgen::proxy::EnrichedFieldSymbol<'s>>,
    {
        /*
         * struct SpanId:
         *     * size 24
         *     * align 8
         */
        let fields = symbol.fields.collect::<Vec<_>>();
        assert_eq!(symbol.name, "SpanId");
        assert_eq!(fields.len(), 2);
        assert_eq!(
            fields[0],
            EnrichedFieldSymbol {
                name: "value".into(),
                kind: EnrichedFieldType::Type {
                    name: creamy_libgen::Path::from_global("u64")
                },
                is_padding: false,
            }
        );
        assert_eq!(
            fields[1],
            EnrichedFieldSymbol {
                name: "__padding0".into(),
                kind: EnrichedFieldType::Array {
                    kind: "u8".into(),
                    len: 16
                },
                is_padding: true,
            }
        );
    }

    fn end_group(&mut self) {}

    fn generate_dispatcher<I>(&mut self, _: I)
    where
        I: Iterator<Item = creamy_libgen::Path>,
    {
    }

    fn generate_group_lifecycle_hook<I>(&mut self, _: I)
    where
        I: Iterator<Item = creamy_libgen::Path>,
    {
    }

    fn generate_plugin_group_table(&mut self, _: creamy_phf::PerfectHashTable) {}

    fn generate_host_group_table(&mut self, _: creamy_phf::PerfectHashTable) {}

    fn generate_host_package(&mut self, _: Vec<u8>) {}

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[test]
fn struct_() -> Result<(), Box<dyn core::error::Error>> {
    let root = setup_root()?;
    let directory = root.as_ref().join(".creamy");
    let protocols = directory.join("definitions");

    let working_dir = std::env::current_dir()?;
    std::fs::copy(
        working_dir.join("tests/xml/system.xml"),
        protocols.join("system.xml"),
    )?;

    let mut library = ProtocolLibrary::new(
        &std::fs::read_to_string(directory.join("manifest.toml"))?,
        root.as_ref().into(),
    );
    library.load_all(&protocols)?;

    let mut codegen = Codegen::new(library, Target::Host);
    codegen.run(&mut Mock)?;
    Ok(())
}
