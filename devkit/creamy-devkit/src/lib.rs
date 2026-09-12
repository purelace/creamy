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

pub mod binrw {
    pub use binrw::*;
}

use std::{fs::ReadDir, path::Path, str::FromStr};

use ::semver::Version;
use compiler::utils::{BString, collections::List, strpool::StringPool};
use creamy_manifest::Manifest;
use creamy_xmlc::{ProtocolDefinition, compile};
use fs_err as fs;

use self::error::Error::TooManyFiles;
pub use crate::error::Error;

/// Represents a compiled binary plugin containing metadata, protocol definitions, and core logic.
#[binrw::binrw]
#[brw(magic = b"CMY!", little)]
#[derive(Debug, PartialEq, Eq)]
pub struct BinaryPlugin {
    /// The semantic version of the plugin.
    #[br(map = |val: BString| Version::from_str(&val).unwrap())]
    #[bw(map = |val: &Version| BString::wrap(val.to_string()))]
    pub version: Version,
    /// The manifest containing plugin metadata.
    pub manifest: Manifest,
    /// The string pool used for shared string references.
    pub pool: StringPool,
    /// A list of compiled protocol definitions.
    pub definitions: List<ProtocolDefinition>,
    /// The raw core logic of the plugin.
    core: List<u8>,
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
    pub fn core(&self) -> &[u8] {
        self.core.as_slice()
    }

    #[cfg(feature = "write")]
    /// Writes the binary plugin data to a file.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](binrw::Error) variant will be returned.
    pub fn write_to<W: std::io::Write + std::io::Seek>(&self, writer: &mut W) -> Result<(), Error> {
        use binrw::BinWrite;
        self.write(writer)?;
        Ok(())
    }

    #[cfg(feature = "read")]
    /// Loads a `BinaryPlugin` from the specified reader.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](binrw::Error) variant will be returned.
    pub fn read_from<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<Self, Error> {
        use binrw::BinRead;
        Ok(Self::read(reader)?)
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
    let definitions = if std::fs::exists(&definitions_path)? {
        let dir = std::fs::read_dir(definitions_path)?;
        compile_protocols(dir, &mut pool)?
    } else {
        List::default()
    };

    Ok(BinaryPlugin {
        version: Version::from_str(env!("CARGO_PKG_VERSION"))?,
        manifest,
        pool,
        definitions,
        core: List::wrap(module),
    })
}

fn compile_protocols(
    dir: ReadDir,
    pool: &mut StringPool,
) -> Result<List<ProtocolDefinition>, Error> {
    let files = dir
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|p| p == "xml"))
        .map(|e| e.path())
        .collect::<Vec<_>>();

    let Ok(len) = u32::try_from(files.len()) else {
        return Err(TooManyFiles);
    };

    let mut protocols = List::with_capacity(len);
    for path in files {
        let content = fs::read_to_string(path)?;
        protocols.push(compile(pool, &content).unwrap());
    }

    Ok(protocols)
}
