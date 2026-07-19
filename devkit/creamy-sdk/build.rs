use creamy_libgen_rs::{Args, script::generate_code};

const MANIFEST: &str = include_str!("manifest.toml");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(outdir) = std::env::var_os("OUT_DIR") else {
        return Ok(());
    };

    generate_code(
        "./",
        outdir,
        MANIFEST,
        Args::default().with_creamy_sdk_path("crate"),
    )
}
