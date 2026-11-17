#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod build;
pub mod cli;
mod init;
mod show;
mod tui;
mod utils;

use clap::Parser;
use creamy_xmlc::SemanticModel;

use self::utils::get_workdir;
use crate::{
    cli::{Command, Validate},
    init::init_template,
    show::execute_show_cmd,
};

pub fn run() -> anyhow::Result<()> {
    let args = cli::Args::parse();
    match args.command {
        Some(command) => match command {
            Command::Init => init_template(get_workdir(None)?),
            Command::Show(list) => execute_show_cmd(list),
            Command::Build { .. } =>
            /* build::build(workdir)*/
            {
                Ok(())
            }
            Command::Validate(args) => validate(args),
        },
        None => Ok(()),
    }
}

fn compile_protocol(file: &str) -> anyhow::Result<SemanticModel> {
    let content = std::fs::read_to_string(file)?;
    match SemanticModel::compile(&content) {
        Ok(m) => Ok(m),
        Err(d) => {
            d.print(file, &content);
            panic!();
        }
    }
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
