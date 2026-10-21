#![no_std]

include!(concat!(env!("OUT_DIR"), "/Math.rs"));

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use creamy_sdk::{Sender, api::Plugin, declare_plugin, error, info, stream::StreamReader};
use smol_str::SmolStr;

use self::generated::{
    dispatcher::{GroupLifecycleHook, MessageHandler},
    shell::setup::{
        AddCommandArgument, ExecuteCommand, RegisterCommand, SetCommandName,
        register_command::RegisterCommandContent,
    },
};

declare_plugin!(MathPlugin, generated);

struct MathPlugin {
    sender: Sender,
    name: SmolStr,
    arguments: Vec<SmolStr>,

    name_stream: Option<StreamReader<SetCommandName>>,
    arg_stream: Option<StreamReader<AddCommandArgument>>,
}

impl Plugin for MathPlugin {
    fn init(sender: Sender) -> Option<Self> {
        Some(Self {
            sender,
            name: SmolStr::default(),
            arguments: Vec::with_capacity(16),
            name_stream: None,
            arg_stream: None,
        })
    }
}

impl GroupLifecycleHook for MathPlugin {
    fn on_setup_group_enabled(&mut self) {
        self.sender
            .send_stream::<RegisterCommand>(&RegisterCommandContent::new(Box::from(
                "print".as_bytes(),
            )));
    }

    fn on_setup_group_disabled(&mut self) {}
}

impl MathPlugin {
    pub fn read_name_command(
        &mut self,
        message: SetCommandName,
        mut reader: StreamReader<SetCommandName>,
    ) {
        if let Some(result) = reader.accept_message(&message) {
            match result {
                Ok(mut c) => {
                    let bytes = c.read().read_name();
                    self.name = SmolStr::from(str::from_utf8(bytes).unwrap());
                }
                Err(err) => error!("{err}"),
            }
        } else {
            self.name_stream = Some(reader);
        }
    }

    pub fn read_add_argument_command(
        &mut self,
        message: AddCommandArgument,
        mut reader: StreamReader<AddCommandArgument>,
    ) {
        if let Some(result) = reader.accept_message(&message) {
            match result {
                Ok(mut c) => {
                    let bytes = c.read().read_name();
                    let str = SmolStr::from(str::from_utf8(bytes).unwrap());
                    self.arguments.push(str);
                }
                Err(err) => error!("{err}"),
            }
        } else {
            self.arg_stream = Some(reader);
        }
    }
}

impl MessageHandler for MathPlugin {
    fn handle_set_command_name(&mut self, message: SetCommandName) {
        if let Some(reader) = self.name_stream.take() {
            self.read_name_command(message, reader);
        } else {
            let reader = StreamReader::<SetCommandName>::default();
            self.read_name_command(message, reader);
        }
    }

    fn handle_add_command_argument(&mut self, message: AddCommandArgument) {
        if let Some(reader) = self.arg_stream.take() {
            self.read_add_argument_command(message, reader);
        } else {
            let reader = StreamReader::<AddCommandArgument>::default();
            self.read_add_argument_command(message, reader);
        }
    }

    fn handle_execute_command(&mut self, _: ExecuteCommand) {
        if self.name.is_empty() {
            error!("Command name is empty");
            return;
        }

        match self.name.as_str() {
            "print" => info!("How are you?"),
            _ => unreachable!(),
        }

        self.name = SmolStr::default();
        self.arguments.clear();
    }

    fn handle_unknown_message(
        &mut self,
        _dispatch_value: u32,
        _message: creamy_sdk::message::UntypedMessage,
    ) {
    }
}
