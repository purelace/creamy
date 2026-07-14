#![allow(clippy::cast_possible_truncation)]

mod build;
mod cli;
mod generate;
mod init;
mod show;
mod utils;

use clap::Parser;
use creamy_utils::strpool::StringPool;
use creamy_xmlc::{ProtocolDefinition, compile};

use self::utils::get_workdir;
use crate::{
    cli::{Args, Command, Validate},
    //generate::generate_headers,
    init::init_template,
    show::execute_show_cmd,
};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Some(command) => match command {
            Command::Init => init_template(get_workdir(None)?),
            //Command::Generate {
            //    xml_file,
            //    output,
            //    rewrite,
            //} => generate_headers(xml_file, get_workdir(output)?, rewrite),
            Command::Show(list) => execute_show_cmd(list),
            Command::Build { workdir } => build::build(workdir),
            Command::Validate(args) => validate(args),
        },
        None => Ok(()),
    }
}

fn compile_protocol(pool: &mut StringPool, xml_file: String) -> anyhow::Result<ProtocolDefinition> {
    let content = std::fs::read_to_string(xml_file)?;
    Ok(compile(pool, &content).unwrap())
}

#[allow(unused)]
#[allow(clippy::unnecessary_wraps)]
fn validate(validate: Validate) -> anyhow::Result<()> {
    match validate {
        Validate::Definition { file } => {}
        Validate::Manifest { file } => {}
        Validate::Config { file } => {}
    }

    Ok(())
}
