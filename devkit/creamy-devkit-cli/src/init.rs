use std::{io::Write, path::Path};

fn manifest_template(input: &UserInput) -> String {
    format!(
        r#"
[package]
id = "{id}"
name = "{name}"
version = "0.1.0"
description = "{desc}"
repository = "{repo}"
authors = [""]

# [core]
# path = ""
# runtime = ""

# [[protocols]]
# name = "compositor.wm"
# version = "0.1"
# group = 1
"#,
        id = input.id,
        name = input.name,
        desc = input.desc,
        repo = input.repo,
    )
}

pub fn init_template(workdir: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let input = read_user_manifest()?;
    let workdir = workdir.as_ref();
    let path = workdir.join(&input.name);
    std::fs::create_dir(&path)?;

    std::fs::create_dir(path.join("assets"))?;
    std::fs::create_dir(path.join("config"))?;
    std::fs::create_dir(path.join("definitions"))?;
    std::fs::create_dir(path.join("plugin"))?;

    let mut manifest_file = std::fs::File::create(path.join("manifest.toml"))?;
    manifest_file.write_all(manifest_template(&input).as_bytes())?;

    if let Some(command) = input.exec {
        let parts = command.split_whitespace().collect::<Vec<&str>>();
        let name = &parts[0];
        let args = if parts.is_empty() { &[] } else { &parts[1..] };
        std::process::Command::new(name)
            .current_dir(path.join("plugin"))
            .args(args)
            .spawn()?
            .wait()?;
    }
    Ok(())
}

fn read_user_manifest() -> Result<UserInput, Box<dyn std::error::Error>> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let id = rl.readline(">> ID: ")?;
    let name = rl.readline(">> Name: ")?;
    let desc = rl.readline(">> Description: ")?;
    let repo = rl.readline(">> Repository: ")?;
    let exec = &rl.readline(">> Execute after init [y/N]: ")?;
    let exec = if exec == "y" {
        Some(rl.readline(">> Command: ")?)
    } else {
        None
    };

    //print!("Authors (separate by comma): ");
    //stdout.flush()?;
    //let mut authors = String::with_capacity(256);
    //stdin.read_line(&mut authors)?;
    //let authors = authors
    //    .split(',')
    //    .map(str::trim)
    //    .map(str::to_string)
    //    .collect();

    Ok(UserInput {
        id,
        name,
        desc,
        repo,
        _authors: vec![],
        exec,
    })
}

pub struct UserInput {
    id: String,
    name: String,
    desc: String,
    repo: String,
    _authors: Vec<String>,
    exec: Option<String>,
}
