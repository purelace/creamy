use std::{path::Path, str::FromStr};

use creamy_manifest::Manifest;
use creamy_protocol_model::definition::ProtocolModel;
use creamy_utils::strpool::StringPool;
use creamy_xmlc::compile;
use fs_err as fs;
use semver::Version;

use crate::{BinaryPlugin, Error};

/// Compiles a plugin from a directory and a core module into a `BinaryPlugin`.
///
/// # Errors
///
/// Returns an error if:
/// * The `.creamy` directory is missing.
/// * The `manifest.toml` is malformed or missing.
/// * There are issues reading the filesystem or parsing XML definitions.
/// * The version string is invalid.
pub fn compile_to_binary(
    plugin_dir: impl AsRef<Path>,
    core: Box<[u8]>,
) -> Result<BinaryPlugin, Error> {
    let mut pool = StringPool::default();

    let plugin_dir = plugin_dir.as_ref();
    let creamy_dir = plugin_dir.join(".creamy");
    if !fs::exists(&creamy_dir)? {
        return Err(Error::MissingDirectory);
    }

    let manifest_path = creamy_dir.join("manifest.toml");
    let manifest_file = fs::read_to_string(manifest_path)?;
    let manifest = Manifest::read_manifest(&manifest_file)?;

    let definitions_path = creamy_dir.join("definitions");
    let models = if fs::exists(&definitions_path)? {
        let dir = fs::read_dir(definitions_path)?;
        compile_protocols(dir, &mut pool)?
    } else {
        vec![]
    };

    Ok(BinaryPlugin {
        version: Version::from_str(env!("CARGO_PKG_VERSION")).expect("Unreachable"),
        manifest,
        pool,
        models,
        core,
    })
}

fn compile_protocols(dir: fs::ReadDir, pool: &mut StringPool) -> Result<Vec<ProtocolModel>, Error> {
    let files = dir
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|p| p == "xml"))
        .map(|e| e.path())
        .collect::<Vec<_>>();

    let Ok(len) = u32::try_from(files.len()) else {
        return Err(Error::TooManyFiles);
    };

    let mut protocols = Vec::with_capacity(len as usize);
    for path in files {
        let content = fs::read_to_string(path)?;
        protocols.push(compile(pool, &content).unwrap());
    }

    Ok(protocols)
}
