#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod memory;
use std::num::NonZeroU8;

use console::style;
use creamy_utils::strpool::StringPool;
use creamy_xmlc::{
    ProtocolDefinition, StringPoolResolver,
    constraints::HEADER_BYTES,
    model::symbols::{FieldSymbol, FieldType, Type},
};

use crate::memory::{ArrayField, MemoryReport, SimpleField};

const STRUCT_COLOR: (u8, u8, u8) = (78, 201, 176);
const GROUP_COLOR: (u8, u8, u8) = (219, 219, 169);

fn print_header(def: &ProtocolDefinition, pool: &StringPool) {
    let name = style(def.name().resolve(pool)).green().bold();
    let version = style(format!("v{}", def.version())).yellow().bold();
    println!("— {name} | {version}");
}

fn print_item(group_idx: u8, message_idx: u8, name: &str, name_color: (u8, u8, u8)) {
    let index = if group_idx == 0 {
        message_idx.to_string()
    } else {
        format!("{group_idx}:{message_idx}")
    };
    let name = style(name)
        .true_color(name_color.0, name_color.1, name_color.2)
        .bold();
    println!("* [{index}] {name}");
}

pub fn show_all_messages(def: &ProtocolDefinition, pool: &StringPool) {
    print_header(def, pool);
    println!("+ List of messages");

    for (group_idx, group) in def.groups().iter().enumerate() {
        for (message_idx, message) in def.messages_slice(group.messages()).iter().enumerate() {
            let name_id = message.name();
            print_item(
                group_idx as u8 + 1,
                message_idx as u8,
                name_id.resolve(pool),
                STRUCT_COLOR,
            );
        }
    }
}

pub fn show_one_group_by_idx(def: &ProtocolDefinition, pool: &StringPool, idx: NonZeroU8) {
    let group = def.groups()[idx.get() as usize - 1];
    let group_name_id = group.name();
    let group_name = group_name_id.resolve(pool);
    println!("+ List of messages ({group_name})");
    print_header(def, pool);

    let slice = def.messages_slice(group.messages());
    for (message_idx, message) in slice.iter().enumerate() {
        let id = message.name();
        print_item(idx.get(), message_idx as u8, id.resolve(pool), STRUCT_COLOR);
    }
}

pub fn show_one_group_by_name(def: &ProtocolDefinition, pool: &mut StringPool, name: &str) {
    let name_id = pool.get_id(name);
    if let Some(idx) = def
        .groups()
        .iter()
        .enumerate()
        .find(|(_, g)| g.name() == name_id)
        .map(|(idx, _)| idx)
    {
        show_one_group_by_idx(def, pool, NonZeroU8::new(idx as u8 + 1).unwrap());
    } else {
        panic!("todo")
    }
}

pub fn show_all_groups(def: &ProtocolDefinition, pool: &StringPool) {
    println!("+ List of groups");
    print_header(def, pool);

    for (idx, group) in def.groups().iter().enumerate() {
        let idx = idx + 1;
        let id = group.name();
        print_item(0, idx as u8, id.resolve(pool), GROUP_COLOR);
    }
}

pub fn show_memory_layout(
    def: &ProtocolDefinition,
    pool: &StringPool,
    flat: bool,
    group: NonZeroU8,
    msg: u8,
) {
    let group = &def.groups()[group.get() as usize - 1];
    let messages = def.messages_slice(group.messages());
    let message = messages[msg as usize];
    let fields = def.fields_slice(message.fields());

    let mut report = MemoryReport::new(message.name().resolve(pool));
    if flat {
        make_flat_report(def, &mut report, fields, pool);
    } else {
        make_report(def, &mut report, fields, pool);
    }
    let meta = ProtocolDefinition::get_struct_meta(fields).unwrap();
    let finished_report = report.finish(
        meta.size().value() as usize,
        meta.align().value() as usize,
        ProtocolDefinition::get_struct_paddings(fields) as usize,
    );
    finished_report.print_tree();
}

fn make_flat_report(
    def: &ProtocolDefinition,
    report: &mut MemoryReport,
    fields: &[FieldSymbol],
    pool: &StringPool,
) {
    let mut total_size = HEADER_BYTES;
    let mut paddings = 0u8;

    let tt = def.table();
    for field in fields {
        let tid = field.type_id();
        let size = tid.size().value();
        let align = tid.align().value();

        let padding = (align - (total_size % align)) % align;
        total_size += padding + size;

        if padding != 0 {
            report.add_array_field(ArrayField {
                name: format!("_padding{paddings}"),
                kind: "u8".to_string(),
                size: padding as usize,
                align: 1,
                is_padding: true,
            });
            paddings += 1;
        }

        match field.kind() {
            FieldType::Type(tid) => match tt.get_type(tid) {
                Type::Array(_) => unreachable!(),
                Type::Struct(symbol) => {
                    let fields = def.fields_slice(symbol.fields());
                    make_flat_report(def, report, fields, pool);
                }
                Type::Numeric(_) | Type::Enum(_) => {
                    report.add_field(SimpleField {
                        name: field.name().resolve(pool).to_string(),
                        kind: tt.name_of_type(tid).resolve(pool).to_string(),
                        size: size as usize,
                        align: align as usize,
                    });
                }
                Type::Flags(flags_symbol) => todo!(),
                Type::Bitset(bitset_symbol) => todo!(),
            },
            FieldType::Array(array) => {
                report.add_array_field(ArrayField {
                    name: field.name().resolve(pool).to_string(),
                    kind: tt.name_of_type(array.kind()).resolve(pool).to_string(),
                    size: array.len().value() as usize,
                    align: align as usize,
                    is_padding: false,
                });
            }
        }
    }
}

fn make_report(
    def: &ProtocolDefinition,
    report: &mut MemoryReport,
    fields: &[FieldSymbol],
    pool: &StringPool,
) {
    let mut total_size = HEADER_BYTES;
    let mut paddings = 0u8;

    let tt = def.table();
    for field in fields {
        let tid = field.type_id();
        let size = tid.size().value();
        let align = tid.align().value();

        let padding = (align - (total_size % align)) % align;
        total_size += padding + size;

        if padding != 0 {
            report.add_array_field(ArrayField {
                name: format!("_padding{paddings}"),
                kind: "u8".to_string(),
                size: padding as usize,
                align: 1,
                is_padding: true,
            });
            paddings += 1;
        }

        match tt.get_type(tid) {
            Type::Array(_) => unreachable!(),
            Type::Struct(sym) => {
                report.start_report();
                let fields = def.fields_slice(sym.fields());
                make_report(def, report, fields, pool);
                report.end_report(SimpleField {
                    name: field.name().resolve(pool).to_string(),
                    kind: tt.name_of_type(tid).resolve(pool).to_string(),
                    size: size as usize,
                    align: align as usize,
                });
            }
            Type::Numeric(_) | Type::Enum(_) => {
                report.add_field(SimpleField {
                    name: field.name().resolve(pool).to_string(),
                    kind: tt.name_of_type(tid).resolve(pool).to_string(),
                    size: size as usize,
                    align: align as usize,
                });
            }
            Type::Flags(flags_symbol) => todo!(),
            Type::Bitset(bitset_symbol) => todo!(),
        }
    }
}
