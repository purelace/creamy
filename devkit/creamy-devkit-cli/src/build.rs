use std::path::Path;

use anyhow::Context;
use creamy_devkit::compile_to_binary;
use fs_err as fs;
use serde::Deserialize;

use crate::utils::get_workdir;

#[derive(Deserialize)]
struct BuildConfiguration {
    command: String,
    input: String,
    output: String,
}

fn build_wasm_module(configuration: &BuildConfiguration) -> anyhow::Result<()> {
    let mut iter = configuration.command.split(' ');
    let command = iter.next().unwrap();
    let args = iter.collect::<Vec<_>>();
    std::process::Command::new(command)
        .args(args)
        .spawn()?
        .wait()?;
    Ok(())
}

fn optimize_wasm_module(
    workdir: &Path,
    output_file: &Path,
    configuration: &BuildConfiguration,
) -> anyhow::Result<()> {
    let wasm_module_path = workdir.join(&configuration.input).canonicalize()?;
    std::process::Command::new("wasm-opt")
        .args([
            wasm_module_path.to_str().unwrap(),
            "-O3",
            "-o",
            output_file.to_str().unwrap(),
        ])
        .spawn()?
        .wait()?;
    Ok(())
}

// TODO: check if `configuration.input` ends with .wasm
pub fn build(workdir: Option<String>) -> anyhow::Result<()> {
    let workdir = get_workdir(workdir)?;

    let build_path = workdir.join("build.toml");
    let content = fs::read_to_string(&build_path)
        .with_context(|| format!("Path: {}", build_path.display()))?;

    let configuration = toml::from_str::<BuildConfiguration>(&content)?;

    let output_path = workdir.join(&configuration.output);
    fs::create_dir_all(&output_path).with_context(|| format!("Path: {}", output_path.display()))?;

    let output_path = output_path.canonicalize()?;

    build_wasm_module(&configuration)?;

    let output_file = workdir.join(&output_path).join("module.optimized.wasm");
    optimize_wasm_module(&workdir, &output_file, &configuration)?;

    let binary = compile_to_binary(workdir, fs::read(&output_file)?)?;
    let out = output_path
        .join(binary.manifest().name())
        .with_extension("cmy");
    binary
        .write_to_file(out)
        .map_err(anyhow::Error::from_boxed)?;
    Ok(())
}
