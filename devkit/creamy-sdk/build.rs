use creamy_libgen_rs::{Args, script::generate_code};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(outdir) = std::env::var_os("OUT_DIR") else {
        return Ok(());
    };

    let manifest = r#"
[package]
id = "org.creamy.sdk"
name = "system"
version = "1.0.0"
description = "Builtin package"
repository = "https://github.com/purelace/creamy"
authors = [ "selrisu <myirisuchan@gmail.com>" ]

[core]
path = "core.wasm"
runtime = "wasm"

[protocols]
system = { version = "1.0", groups=["builtin"]}
"#;

    generate_code(
        "./",
        outdir,
        manifest,
        Args::default().with_creamy_sdk_path("crate"),
    )
}
