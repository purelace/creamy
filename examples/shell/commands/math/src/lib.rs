#![no_std]

include!(concat!(env!("OUT_DIR"), "/Math.rs"));

extern crate alloc;
mod reader;
mod writer;

use alloc::vec::Vec;

use creamy_sdk::{
    Sender,
    api::Plugin,
    declare_plugin, error, info,
    stream::{StreamId, StreamMessage, StreamReader, StreamWriter},
};
use smol_str::SmolStr;

use self::{
    generated::{
        dispatcher::{GroupLifecycleHook, MessageHandler},
        shell::setup::{AddCommandArgument, ExecuteCommand, SetCommandName},
    },
    reader::{ArgumentReader, NameReader},
    writer::StringWriter,
};

declare_plugin!(MathPlugin, generated);

struct MathPlugin {
    sender: Sender,
    name: SmolStr,
    arguments: Vec<SmolStr>,

    name_stream: Option<StreamReader<NameReader>>,
    arg_stream: Option<StreamReader<ArgumentReader>>,
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
        self.sender.send_stream(
            StreamWriter::new(StringWriter::default(), StreamId::new(10)),
            "print",
        );
    }

    fn on_setup_group_disabled(&mut self) {}
}

impl MathPlugin {
    pub fn read_name_command(
        &mut self,
        message: SetCommandName,
        mut reader: StreamReader<NameReader>,
    ) {
        match reader.read(message) {
            Ok(result) => {
                if result {
                    match reader.into_reader().into_string() {
                        Ok(str) => self.name = str,
                        Err(err) => error!("{err}"),
                    }
                } else {
                    self.name_stream = Some(reader);
                }
            }
            Err(err) => error!("{err}"),
        }
    }

    pub fn read_add_argument_command(
        &mut self,
        message: AddCommandArgument,
        mut reader: StreamReader<ArgumentReader>,
    ) {
        match reader.read(message) {
            Ok(result) => {
                if result {
                    match reader.into_reader().into_string() {
                        Ok(str) => self.arguments.push(str),
                        Err(err) => error!("{err}"),
                    }
                } else {
                    self.arg_stream = Some(reader);
                }
            }
            Err(err) => error!("{err}"),
        }
    }
}

impl MessageHandler for MathPlugin {
    fn handle_set_command_name(&mut self, message: SetCommandName) {
        if let Some(reader) = self.name_stream.take() {
            self.read_name_command(message, reader);
        } else {
            let reader = StreamReader::new(message.stream_id(), NameReader::default());
            self.read_name_command(message, reader);
        }
    }

    fn handle_add_command_argument(&mut self, message: AddCommandArgument) {
        if let Some(reader) = self.arg_stream.take() {
            self.read_add_argument_command(message, reader);
        } else {
            let reader = StreamReader::new(message.stream_id(), ArgumentReader::default());
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
