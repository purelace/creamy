use creamy_libgen_rs::{Args, script::generate_code};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(outdir) = std::env::var_os("OUT_DIR") else {
        return Ok(());
    };

    generate_code(
        ".creamy/definitions",
        outdir,
        &std::fs::read_to_string(".creamy/manifest.toml")?,
        Args::default(),
    )
}
