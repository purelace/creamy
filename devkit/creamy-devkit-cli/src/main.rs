#![allow(clippy::cast_possible_truncation)]

mod cli;
mod generate;
mod init;
mod show;

use std::path::PathBuf;

use clap::Parser;
use creamy_devkit::compile_to_binary;
use creamy_utils::strpool::StringPool;
use creamy_xmlc::{ProtocolDefinition, compile};

use crate::{
    cli::{Args, Command, Validate},
    //generate::generate_headers,
    init::init_template,
    show::execute_show_cmd,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            Command::Build { workdir, output } => build(workdir, output),
            Command::Validate(args) => validate(args),
        },
        None => Ok(()),
    }
}

fn compile_protocol(
    pool: &mut StringPool,
    xml_file: String,
) -> Result<ProtocolDefinition, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(xml_file)?;
    Ok(compile(pool, &content).unwrap())
}

fn get_workdir(workdir: Option<String>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(if let Some(workdir) = workdir {
        PathBuf::from(workdir)
    } else {
        std::env::current_dir()?
    })
}

fn build(
    workdir: Option<String>,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let workdir = get_workdir(workdir)?;
    let outdir = if let Some(output) = output {
        PathBuf::from(output)
    } else {
        workdir.clone()
    };

    let binary = compile_to_binary(workdir)?;
    let out = outdir.join(binary.name()).with_extension("cmy");
    binary.write_to_file(out)?;
    Ok(())
}

#[allow(unused)]
#[allow(clippy::unnecessary_wraps)]
fn validate(validate: Validate) -> Result<(), Box<dyn std::error::Error>> {
    match validate {
        Validate::Definition { file } => {}
        Validate::Manifest { file } => {}
        Validate::Config { file } => {}
    }

    Ok(())
}
