mod error;

pub mod compiler {
    pub use creamy_xmlc::*;
}

pub mod manifest {
    pub use creamy_manifest::*;
}

pub mod semver {
    pub use semver::*;
}

use std::{fs::ReadDir, path::Path, str::FromStr};

use ::semver::Version;
use creamy_binary::{BinRead, BinWrite};
use creamy_manifest::Manifest;
use creamy_xmlc::{
    compile,
    model::{definition::ProtocolModel, strpool::StringPool},
};
use fs_err as fs;

use self::error::Error::TooManyFiles;
pub use crate::error::Error;

/// Represents a compiled binary plugin containing metadata, protocol definitions, and core logic.
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
#[magic(b"CMY!")]
pub struct BinaryPlugin {
    /// The semantic version of the plugin.
    pub version: Version,
    /// The manifest containing plugin metadata.
    pub manifest: Manifest,
    /// The string pool used for shared string references.
    pub pool: StringPool,
    /// A list of compiled protocol models.
    pub models: Vec<ProtocolModel>,
    /// The raw core logic of the plugin.
    core: Vec<u8>,
}

impl BinaryPlugin {
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    #[must_use]
    pub const fn string_pool(&self) -> &StringPool {
        &self.pool
    }

    #[must_use]
    pub const fn core(&self) -> &[u8] {
        self.core.as_slice()
    }
}

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
    module: Vec<u8>,
) -> Result<BinaryPlugin, Error> {
    let mut pool = StringPool::default();

    let plugin_dir = plugin_dir.as_ref();
    let creamy_dir = plugin_dir.join(".creamy");
    if !std::fs::exists(&creamy_dir)? {
        return Err(Error::MissingDirectory);
    }

    let manifest_path = creamy_dir.join("manifest.toml");
    let manifest_file = std::fs::read_to_string(manifest_path)?;
    let manifest = Manifest::read_manifest(&manifest_file)?;

    let definitions_path = creamy_dir.join("definitions");
    let models = if std::fs::exists(&definitions_path)? {
        let dir = std::fs::read_dir(definitions_path)?;
        compile_protocols(dir, &mut pool)?
    } else {
        vec![]
    };

    Ok(BinaryPlugin {
        version: Version::from_str(env!("CARGO_PKG_VERSION")).expect("Unreachable"),
        manifest,
        pool,
        models,
        core: module,
    })
}

fn compile_protocols(dir: ReadDir, pool: &mut StringPool) -> Result<Vec<ProtocolModel>, Error> {
    let files = dir
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|p| p == "xml"))
        .map(|e| e.path())
        .collect::<Vec<_>>();

    let Ok(len) = u32::try_from(files.len()) else {
        return Err(TooManyFiles);
    };

    let mut protocols = Vec::with_capacity(len as usize);
    for path in files {
        let content = fs::read_to_string(path)?;
        protocols.push(compile(pool, &content).unwrap());
    }

    Ok(protocols)
}
