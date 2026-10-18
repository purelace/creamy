use std::env;

use clap::{CommandFactory, ValueEnum};
use clap_complete::{Shell, generate_to};

include!("src/cli.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(outdir) = env::var_os("OUT_DIR") else {
        return Ok(());
    };

    let mut cmd = Args::command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "creamy", &outdir)?;
    }
    Ok(())
}
