#![allow(clippy::missing_errors_doc)]

use std::{io::Write, path::Path};

use creamy_utils::strpool::StringPool;
use creamy_xmlc::{
    FinishedTypeTable, ProtocolDefinition, StringPoolResolver,
    constraints::HEADER_BYTES,
    model::{
        definition::compute_layout,
        symbols::{FieldSymbol, FieldType, Remainder, Type},
    },
};

pub fn generate(
    outdir: impl AsRef<Path>,
    pool: &StringPool,
    definition: &ProtocolDefinition,
    rewrite: bool,
) -> Result<(), std::io::Error> {
    let outdir = outdir.as_ref().join(definition.name().resolve(pool));
    if !std::fs::exists(&outdir)? {
        std::fs::create_dir(&outdir)?;
    }

    //writeln!(types_header, "typedef int8_t bool")?;

    definition.group_iter::<std::io::Error>(|group, messages, types| {
        let header_path = outdir.join(group.name().resolve(pool)).with_extension("h");
        let mut header = if rewrite {
            std::fs::File::options()
                .truncate(rewrite)
                .write(true)
                .create(true)
                .open(header_path)
        } else {
            std::fs::File::create(header_path)
        }?;

        writeln!(header, "#pragma once")?;
        writeln!(header, "#include <creamy/types.h>")?;
        writeln!(header, "#include <creamy/limits.h>")?;
        writeln!(header)?;

        for ty in types {
            match ty {
                Type::Numeric(_) | Type::Array(_) => {}
                Type::Enum(sym) => {
                    let enum_name = sym.name().resolve(pool);
                    writeln!(header, "typedef {} {enum_name};", sym.repr())?;

                    writeln!(header, "typedef enum {enum_name}_Values {{")?;
                    for variant in definition.variants_slice(sym.variants()) {
                        writeln!(header, "    {} = {},", variant.ident().resolve(pool), variant.value())?;
                    }
                    writeln!(header, "}} {enum_name}_Values;\n")?;
                }
                Type::Struct(sym) => {
                    let struct_name = sym.name().resolve(pool);
                    writeln!(header, "typedef struct {struct_name} {{")?;
                    write_fields(
                        false,
                        0,
                        Remainder::new(None),
                        &mut header,
                        definition.table(),
                        pool,
                        definition.fields_slice(sym.fields()),
                    )?;
                    writeln!(header, "}} {struct_name};\n")?;
                }
                Type::Flags(flags_symbol) => {},
                Type::Bitset(bitset_symbol) => {},
            }
        }

        for message in messages {
            let message_name = message.name().resolve(pool);
            writeln!(header, "typedef struct {message_name} {{")?;
            writeln!(header, "    /* -------- HEADER -------- */")?;
            writeln!(header, "    u8 dst;")?;
            writeln!(header, "    u8 group;")?;
            writeln!(header, "    u8 src;")?;
            writeln!(header, "    u8 kind;")?;

            writeln!(header, "    /* -------- PAYLOAD -------- */")?;
            write_fields(
                true,
                HEADER_BYTES,
                message.remainder(),
                &mut header,
                definition.table(),
                pool,
                definition.fields_slice(message.fields()),
            )?;
            writeln!(header, "}} {message_name};\n")?;

            writeln!(header, r#"_Static_assert(sizeof(struct {message_name}) == CMY_MESSAGE_SIZE, "Invalid message size.");"#)?;
            writeln!(header)?;
        }

        Ok(())
    })
}

fn write_fields<W: Write>(
    add_remainder: bool,
    reserved: u8,
    remainder: Remainder,
    header: &mut W,
    table: &FinishedTypeTable,
    pool: &StringPool,
    fields: &[FieldSymbol],
) -> Result<(), std::io::Error> {
    let mut paddings = 0;
    let total_size = compute_layout(reserved, fields, |f, s| {
        if s.padding != 0 {
            writeln!(header, "    u8 __padding{paddings}[{}];", s.padding)?;
            paddings += 1;
        }

        match f.kind() {
            FieldType::Type(sym) => {
                let name = table.get_type(sym).ident();
                writeln!(
                    header,
                    "    {} {};",
                    name.resolve(pool),
                    f.name().resolve(pool),
                )
            }
            FieldType::Array(array) => {
                let name = table.get_type(array.kind()).ident();
                writeln!(
                    header,
                    "    {} {}[{}];",
                    name.resolve(pool),
                    f.name().resolve(pool),
                    array.len().value(),
                )
            }
        }
    })?;

    if add_remainder {
        let diff = 32 - total_size;
        if diff != 0 {
            write!(header, "    u8 ")?;
            if let Some(remainder) = remainder.into_inner() {
                write!(header, "{}", remainder.resolve(pool))
            } else {
                write!(header, "__padding{paddings}")
            }?;
            writeln!(header, "[{diff}];")?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use creamy_utils::strpool::StringPool;
    use creamy_xmlc::compile;

    use crate::generate;

    #[test]
    fn test() {
        let content = std::fs::read_to_string(
            "/mnt/ssd/fusionwm/creamy/devkit/creamy-xmlc/tests/success.xml",
        )
        .unwrap();
        let mut pool = StringPool::default();
        let protocol = compile(&mut pool, &content).unwrap();

        generate("/tmp/cmy_msg", &pool, &protocol, true).unwrap();
    }
}
