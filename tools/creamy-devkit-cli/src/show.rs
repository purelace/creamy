use creamy_xmlc::model::{
    definition::ProtocolModel,
    strpool::{StringPool, StringPoolIntern},
};

use crate::{
    cli::{ShowCommand, StringOrNumber},
    compile_protocol,
    tui::{show_all_messages, show_memory_layout},
};

pub fn execute_show_cmd(show: ShowCommand) -> anyhow::Result<()> {
    let mut pool = StringPool::default();
    match show {
        ShowCommand::Layout {
            flat,
            message,
            xml_file,
        } => {
            let model = compile_protocol(&mut pool, xml_file)?;
            let message_index = get_message_index(&model, &mut pool, message).unwrap();
            show_memory_layout(&model, &pool, flat, message_index);
        }
        ShowCommand::Messages { xml_file } => {
            let def = compile_protocol(&mut pool, xml_file)?;
            show_all_messages(&def, &pool);
        }
    }

    Ok(())
}

fn get_message_index(
    model: &ProtocolModel,
    pool: &mut StringPool,
    key: StringOrNumber,
) -> Option<u8> {
    match key {
        StringOrNumber::String(name) => {
            let name_id = name.intern(pool);
            model
                .messages()
                .iter()
                .enumerate()
                .find(|(_, m)| m.ident() == name_id)
                .map(|(i, _)| i as u8)
        }
        StringOrNumber::Number(value) => Some(value),
    }
}
