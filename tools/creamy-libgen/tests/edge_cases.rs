use std::{fs::File, io::Write};

use creamy_libgen::prelude::*;
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

    fn generate_single_message<I>(&mut self, _: EnrichedSingleMessageSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedFieldSymbol<'s>>,
    {
    }

    fn generate_stream_message<F, C>(&mut self, _: EnrichedStreamMessageSymbol<'s, F, C>)
    where
        F: SymbolIterator<EnrichedFieldSymbol<'s>>,
        C: SymbolIterator<EnrichedStreamFieldSymbol<'s>>,
    {
    }

    #[allow(clippy::too_many_lines)]
    fn generate_bitset<I>(&mut self, bitset: EnrichedBitsetSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedBitsetValueSymbol<'s>>,
    {
        let values = bitset.values.collect::<Vec<_>>();
        pretty_assertions::assert_eq!(bitset.name, "TypeMeta");
        pretty_assertions::assert_eq!(
            values.as_slice(),
            [
                /* 10000000_000000000000000000000000 */
                EnrichedBitsetValueSymbol {
                    bytes: 1,
                    bits: 1,
                    read_window_bytes: 1,
                    shift: 7,
                    start_pos: 0,
                    end_pos: 1,
                    backing_type: PrimitiveRepr::U8,
                    repr: PrimitiveRepr::U8,
                    name: "is_valid"
                },
                /* 01000000_000000000000000000000000 */
                EnrichedBitsetValueSymbol {
                    bytes: 1,
                    bits: 1,
                    read_window_bytes: 1,
                    shift: 6,
                    start_pos: 0,
                    end_pos: 1,
                    backing_type: PrimitiveRepr::U8,
                    repr: PrimitiveRepr::U8,
                    name: "is_const"
                },
                /* 00111000_000000000000000000000000 */
                EnrichedBitsetValueSymbol {
                    bytes: 1,
                    bits: 3,
                    read_window_bytes: 1,
                    shift: 3,
                    start_pos: 0,
                    end_pos: 1,
                    backing_type: PrimitiveRepr::U8,
                    repr: PrimitiveRepr::U16,
                    name: "align"
                },
                /* 0000011111000000_0000000000000000 */
                EnrichedBitsetValueSymbol {
                    bytes: 1,
                    bits: 5,
                    read_window_bytes: 2,
                    shift: 6,
                    start_pos: 0,
                    end_pos: 2,
                    backing_type: PrimitiveRepr::U16,
                    repr: PrimitiveRepr::U16,
                    name: "size"
                },
                /* 00000000_00111111_0000000000000000 */
                EnrichedBitsetValueSymbol {
                    bytes: 1,
                    bits: 6,
                    read_window_bytes: 1,
                    shift: 0,
                    start_pos: 1,
                    end_pos: 2,
                    backing_type: PrimitiveRepr::U8,
                    repr: PrimitiveRepr::U32,
                    name: "attributes_mask"
                },
                /* 0000000000000000_11110000_00000000 */
                EnrichedBitsetValueSymbol {
                    bytes: 1,
                    bits: 4,
                    read_window_bytes: 1,
                    shift: 4,
                    start_pos: 2,
                    end_pos: 3,
                    backing_type: PrimitiveRepr::I8,
                    repr: PrimitiveRepr::I8,
                    name: "offset_modifier"
                },
                /* 0000000000000000_0000111111110000 */
                EnrichedBitsetValueSymbol {
                    bytes: 1,
                    bits: 8,
                    read_window_bytes: 2,
                    shift: 4,
                    start_pos: 2,
                    end_pos: 4,
                    backing_type: PrimitiveRepr::U16,
                    repr: PrimitiveRepr::U64,
                    name: "custom_type_id"
                },
                /* 000000000000000000000000_00001111 */
                EnrichedBitsetValueSymbol {
                    bytes: 1,
                    bits: 4,
                    read_window_bytes: 1,
                    shift: 0,
                    start_pos: 3,
                    end_pos: 4,
                    backing_type: PrimitiveRepr::U8,
                    repr: PrimitiveRepr::U8,
                    name: "version"
                },
            ]
        );
    }

    fn generate_flags<I>(&mut self, _: EnrichedFlagsSymbol<'s, I>)
    where
        I: SymbolIterator<&'s str>,
    {
    }

    fn generate_enum<I>(&mut self, symbol: EnrichedEnumSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedVariantSymbol<'s>>,
    {
        let variants = symbol.variants.collect::<Vec<_>>();
        assert_eq!(symbol.name, "Level");
        assert_eq!(symbol.repr, PrimitiveRepr::U8);
        assert_eq!(
            variants.as_slice(),
            [
                EnrichedVariantSymbol {
                    name: "Trace",
                    value: VariantValue::Unsigned(0)
                },
                EnrichedVariantSymbol {
                    name: "Debug",
                    value: VariantValue::Unsigned(1)
                },
                EnrichedVariantSymbol {
                    name: "Info",
                    value: VariantValue::Unsigned(2)
                },
                EnrichedVariantSymbol {
                    name: "Warning",
                    value: VariantValue::Unsigned(3)
                },
                EnrichedVariantSymbol {
                    name: "Error",
                    value: VariantValue::Unsigned(4)
                },
            ]
        );
    }

    fn generate_struct<I>(&mut self, symbol: EnrichedStructSymbol<'s, I>)
    where
        I: SymbolIterator<EnrichedFieldSymbol<'s>>,
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
        I: Iterator<Item = Path>,
    {
    }

    fn generate_group_lifecycle_hook<I>(&mut self, _: I)
    where
        I: Iterator<Item = Path>,
    {
    }

    fn generate_plugin_group_table(&mut self, _: PerfectHashTable) {}

    fn generate_host_group_table(&mut self, _: PerfectHashTable) {}

    fn generate_host_package(&mut self, _: Vec<u8>) {}

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

fn setup_env(xml_file: &str) -> Result<(TempDir, Codegen), Box<dyn core::error::Error>> {
    let root = setup_root()?;
    let directory = root.as_ref().join(".creamy");
    let protocols = directory.join("definitions");

    let working_dir = std::env::current_dir()?;
    std::fs::copy(
        working_dir.join("tests/xml").join(xml_file),
        protocols.join(xml_file),
    )?;
    let mut library = ProtocolLibrary::new(
        &std::fs::read_to_string(directory.join("manifest.toml"))?,
        root.as_ref().into(),
    );
    library.load_all(&protocols)?;
    Ok((root, Codegen::new(library, Target::Host)))
}

#[test]
fn test() -> Result<(), Box<dyn core::error::Error>> {
    let (_dir, mut codegen) = setup_env("struct.xml")?;
    codegen.run(&mut Mock)?;
    Ok(())
}
