use std::{num::NonZeroU8, str::FromStr};

use clap::{
    Parser, Subcommand,
    builder::{Styles, styling::AnsiColor},
};

/// Cargo like style
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::BrightGreen.on_default().bold())
    .usage(AnsiColor::BrightGreen.on_default().bold())
    .literal(AnsiColor::BrightCyan.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
#[command(styles = STYLES)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,

    //Generate {
    //    /// Target XML file
    //    xml_file: String,

    //    /// Output directory
    //    #[arg(short, long)]
    //    output: Option<String>,

    //    #[arg(short, long)]
    //    rewrite: bool,
    //},
    #[command(subcommand)]
    Show(ShowCommand),

    Build {
        /// Working directory
        #[arg(short, long)]
        workdir: Option<String>,
    },

    #[command(subcommand)]
    Validate(Validate),
}

#[derive(Debug, Subcommand)]
pub enum Validate {
    Definition { file: String },
    Manifest { file: String },
    Config { file: String },
}

#[derive(Debug, Clone)]
pub enum StringOrNonZeroNumber {
    String(String),
    Number(NonZeroU8),
}

impl FromStr for StringOrNonZeroNumber {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<NonZeroU8>() {
            Ok(Self::Number(num))
        } else {
            Ok(Self::String(s.to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub enum StringOrNumber {
    String(String),
    Number(u8),
}

impl FromStr for StringOrNumber {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<u8>() {
            Ok(Self::Number(num))
        } else {
            Ok(Self::String(s.to_string()))
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ShowCommand {
    /// Show message structure (use --flat to unfold nested structures)
    Layout {
        /// Unfold and display nested structures
        #[arg(short, long)]
        flat: bool,
        /// Group name or ID [1-255]
        group: StringOrNonZeroNumber,
        /// Message name or ID [0-255]
        message: StringOrNumber,
        /// Target XML file
        xml_file: String,
    },
    /// Show all messages within a specific group
    Group {
        /// Group name or ID [1-255]
        group: StringOrNonZeroNumber,
        /// Target XML file
        xml_file: String,
    },
    /// List all available groups
    Groups {
        /// Target XML file
        xml_file: String,
    },
    /// List all available messages
    Messages {
        /// Target XML file
        xml_file: String,
    },
}
