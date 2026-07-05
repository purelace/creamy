use std::path::Path;

use creamy_utils::strpool::StringPool;
use creamy_xmlc::ProtocolCompiler;

pub fn generate_headers(
    file: impl AsRef<Path>,
    output: impl AsRef<Path>,
    rewrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = output.as_ref();
    let file = file.as_ref();
    let content = std::fs::read_to_string(file)?;
    let mut pool = StringPool::default();
    let mut compiler = ProtocolCompiler::new(&mut pool);
    let def = compiler.compile(&content).unwrap();

    creamy_libgen_c::generate(output, &pool, &def, rewrite)?;

    Ok(())
}
