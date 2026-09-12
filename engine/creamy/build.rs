fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(outdir) = std::env::var_os("OUT_DIR") else {
        return Ok(());
    };
    creamy_libgen_rs::script::generate_code("./", outdir, creamy_libgen_rs::Args::engine())
}
