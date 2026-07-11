use std::{ffi::OsStr, path::PathBuf, process::Command};

use creamy_libgen::{Codegen, ProtocolLibrary};

use crate::{Args, RustGen};

pub fn generate_code(
    protocols: impl AsRef<std::path::Path>,
    outdir: impl AsRef<std::path::Path>,
    manifest: &str,
    args: Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let protocols = protocols.as_ref();
    let working_dir = std::env::current_dir()?;
    let protocols: PathBuf = working_dir.join(protocols);

    let mut library = ProtocolLibrary::new(manifest);
    library.load_all(&protocols)?;

    let mut outdir = outdir.as_ref().to_owned();
    outdir.push(library.manifest().name());
    outdir.set_extension("rs");

    let mut codegen = Codegen::new(library);
    let mut rs_gen = RustGen::new(args, std::fs::File::create(&outdir)?);
    codegen.run(&mut rs_gen)?;

    let _ = Command::new("rustfmt").arg(outdir).status();

    for entry in std::fs::read_dir(protocols)?.filter_map(|result| {
        if let Ok(entry) = result
            && entry.path().extension() == Some(OsStr::new("xml"))
        {
            return Some(entry);
        }

        None
    }) {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    Ok(())
}
