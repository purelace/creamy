use creamy_xmlc::SemanticModel;

use crate::{
    cli::{ShowCommand, StringOrNumber},
    compile_protocol,
    tui::{show_all_messages, show_memory_layout},
};

pub fn execute_show_cmd(show: ShowCommand) -> anyhow::Result<()> {
    match show {
        ShowCommand::Layout {
            flat,
            message,
            file,
        } => {
            let model = compile_protocol(&file)?;
            let message_index = get_message_index(&model, message).unwrap();
            show_memory_layout(&model, flat, message_index);
        }
        ShowCommand::Messages { file } => {
            let model = compile_protocol(&file)?;
            show_all_messages(&model);
        }
    }

    Ok(())
}

fn get_message_index(model: &SemanticModel, key: StringOrNumber) -> Option<u8> {
    match key {
        StringOrNumber::String(name) => {
            let name_id = model.pool().try_get_id(&name)?;
            model
                .messages()
                .iter()
                .enumerate()
                .find(|(_, m)| *m.ident() == name_id)
                .map(|(i, _)| i as u8)
        }
        StringOrNumber::Number(value) => Some(value),
    }
}
