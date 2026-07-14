use std::num::NonZeroU8;

use creamy_protocol_tui::{
    show_all_groups, show_all_messages, show_memory_layout, show_one_group_by_idx,
    show_one_group_by_name,
};
use creamy_utils::strpool::StringPool;
use creamy_xmlc::{ProtocolDefinition, StringPoolIntern};

use crate::{
    cli::{ShowCommand, StringOrNonZeroNumber, StringOrNumber},
    compile_protocol,
};

pub fn execute_show_cmd(show: ShowCommand) -> anyhow::Result<()> {
    let mut pool = StringPool::default();
    match show {
        ShowCommand::Group { group, xml_file } => {
            let def = compile_protocol(&mut pool, xml_file)?;
            match group {
                StringOrNonZeroNumber::String(name) => {
                    show_one_group_by_name(&def, &mut pool, &name);
                }
                StringOrNonZeroNumber::Number(idx) => {
                    show_one_group_by_idx(&def, &pool, idx);
                }
            }
        }
        ShowCommand::Layout {
            flat,
            group,
            message,
            xml_file,
        } => {
            let def = compile_protocol(&mut pool, xml_file)?;
            let group_index = get_group_index(&def, &mut pool, group).unwrap();
            let message_index = get_message_index(&def, &mut pool, group_index, message).unwrap();
            show_memory_layout(
                &def,
                &pool,
                flat,
                NonZeroU8::new(group_index + 1).unwrap(),
                message_index,
            );
        }
        ShowCommand::Groups { xml_file } => {
            let def = compile_protocol(&mut pool, xml_file)?;
            show_all_groups(&def, &pool);
        }
        ShowCommand::Messages { xml_file } => {
            let def = compile_protocol(&mut pool, xml_file)?;
            show_all_messages(&def, &pool);
        }
    }

    Ok(())
}

fn get_group_index(
    def: &ProtocolDefinition,
    pool: &mut StringPool,
    key: StringOrNonZeroNumber,
) -> Option<u8> {
    match key {
        StringOrNonZeroNumber::String(name) => {
            let name_id = pool.get_id_or_add(&name);
            def.groups()
                .iter()
                .enumerate()
                .find(|(_, g)| g.name() == name_id)
                .map(|(idx, _)| idx as u8)
        }
        StringOrNonZeroNumber::Number(value) => Some(value.get() - 1),
    }
}

fn get_message_index(
    def: &ProtocolDefinition,
    pool: &mut StringPool,
    group: u8,
    key: StringOrNumber,
) -> Option<u8> {
    match key {
        StringOrNumber::String(name) => {
            let name_id = name.intern(pool);
            let group = &def.groups()[group as usize];
            let messages = def.messages_slice(group.messages());

            messages
                .iter()
                .enumerate()
                .find(|(_, m)| m.name() == name_id)
                .map(|(idx, _)| idx as u8)
        }
        StringOrNumber::Number(value) => Some(value),
    }
}
