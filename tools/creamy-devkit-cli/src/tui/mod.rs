mod memory;

use console::style;
use creamy_xmlc::model::{
    constraints::HEADER_BYTES,
    definition::ProtocolModel,
    layout::compute_layout,
    strpool::{StringPool, StringPoolResolver},
    symbols::{FieldSymbol, FieldType, MessageSymbolType, Type},
};

use crate::tui::memory::{ArrayField, MemoryReport, SimpleField};

const STRUCT_COLOR: (u8, u8, u8) = (78, 201, 176);

fn print_header(model: &ProtocolModel, pool: &StringPool) {
    let name = style(model.name().resolve(pool)).green().bold();
    let version = style(format!("v{}", model.version())).yellow().bold();
    println!("— {name} | {version}");
}

fn print_item(kind: u8, name: &str, name_color: (u8, u8, u8)) {
    let name = style(name)
        .true_color(name_color.0, name_color.1, name_color.2)
        .bold();
    println!("* [{kind}] {name}");
}

pub fn show_all_messages(model: &ProtocolModel, pool: &StringPool) {
    print_header(model, pool);
    println!("+ List of messages");

    for message in model.messages() {
        let name_id = message.ident();
        print_item(message.kind(), name_id.resolve(pool), STRUCT_COLOR);
    }
}

fn get_struct_paddings(fields: &[FieldSymbol]) -> u8 {
    let mut paddings = 0;
    compute_layout(0, None, fields, |_, l| {
        paddings += l.padding;
    });
    paddings
}

pub fn show_memory_layout(model: &ProtocolModel, pool: &StringPool, flat: bool, msg: u8) {
    let messages = model.messages();
    let message = messages[msg as usize];
    let field_range = match message {
        MessageSymbolType::Single(s) => s.fields(),
        MessageSymbolType::Stream(_) => todo!(), //TODO: support stream messages in memory report
    };
    let fields = model.fields_range(field_range);

    let mut report = MemoryReport::new(message.ident().resolve(pool));
    if flat {
        make_flat_report(model, &mut report, fields, pool);
    } else {
        make_report(model, &mut report, fields, pool);
    }
    let meta = ProtocolModel::get_struct_meta(fields).unwrap();
    let finished_report = report.finish(
        meta.size().value() as usize,
        meta.align().size() as usize,
        get_struct_paddings(fields) as usize,
    );
    finished_report.print_tree();
}

fn make_flat_report(
    model: &ProtocolModel,
    report: &mut MemoryReport,
    fields: &[FieldSymbol],
    pool: &StringPool,
) {
    let mut total_size = HEADER_BYTES;
    let mut paddings = 0u8;

    let tt = model.table();
    for field in fields {
        let tid = field.type_id();
        let size = tid.size().value();
        let align = tid.align().size();

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
                    let fields = model.fields_range(symbol.fields());
                    make_flat_report(model, report, fields, pool);
                }
                Type::Numeric(_) | Type::Enum(_) => {
                    report.add_field(SimpleField {
                        name: field.ident().resolve(pool).to_string(),
                        kind: tt.name_of_type(tid).resolve(pool).to_string(),
                        size: size as usize,
                        align: align as usize,
                    });
                }
                Type::Flags(_) => todo!(),
                Type::Bitset(_) => todo!(),
            },
            FieldType::Array(array) => {
                report.add_array_field(ArrayField {
                    name: field.ident().resolve(pool).to_string(),
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
    def: &ProtocolModel,
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
        let align = tid.align().size();

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
                let fields = def.fields_range(sym.fields());
                make_report(def, report, fields, pool);
                report.end_report(SimpleField {
                    name: field.ident().resolve(pool).to_string(),
                    kind: tt.name_of_type(tid).resolve(pool).to_string(),
                    size: size as usize,
                    align: align as usize,
                });
            }
            Type::Numeric(_) | Type::Enum(_) => {
                report.add_field(SimpleField {
                    name: field.ident().resolve(pool).to_string(),
                    kind: tt.name_of_type(tid).resolve(pool).to_string(),
                    size: size as usize,
                    align: align as usize,
                });
            }
            Type::Flags(_) => todo!(),
            Type::Bitset(_) => todo!(),
        }
    }
}
