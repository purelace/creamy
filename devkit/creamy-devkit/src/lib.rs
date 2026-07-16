#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]

mod error;
mod load;
mod write;

use std::{collections::HashMap, ffi::OsString, path::Path, str::FromStr};

use binrw::binrw;
use creamy_manifest::{Arguments, Manifest};
use creamy_utils::{collections::List, strpool::StringPool, version::Version};
use creamy_xmlc::{ProtocolDefinition, compile};
use fs_err as fs;

pub use crate::error::Error;

#[binrw]
#[brw(magic = b"CMY!", little)]
#[derive(Debug)]
pub struct BinaryPlugin {
    version: Version,
    manifest: Manifest,
    pool: StringPool,
    definitions: List<ProtocolDefinition>,
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
}

pub fn compile_to_binary(
    plugin_dir: impl AsRef<Path>,
    module: Vec<u8>,
) -> Result<BinaryPlugin, Error> {
    let plugin_dir = plugin_dir.as_ref();
    let files = fs::read_dir(plugin_dir)?
        .flatten()
        .map(|dir| (dir.file_name(), dir))
        .collect::<HashMap<_, _>>();

    let manifest_file = files
        .get(&OsString::from("manifest.toml"))
        .ok_or(Error::MissingManifest)?;
    let (_, manifest) = compile_manifest(manifest_file)?;

    let mut pool = StringPool::default();

    let definitions = if let Some(entry) = files.get(&OsString::from("definitions")) {
        compile_protocols(entry, &mut pool)?
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

fn compile_manifest(entry: &fs::DirEntry) -> Result<(Arguments, Manifest), Error> {
    if !entry.file_type()?.is_file() {
        return Err(Error::NotAFile("manifest.toml".to_string()));
    }
    let manifest_content = fs::read_to_string(entry.path())?;

    Ok(Manifest::read_manifest(&manifest_content)?)
}

fn compile_protocols(
    entry: &fs::DirEntry,
    pool: &mut StringPool,
) -> Result<List<ProtocolDefinition>, Error> {
    if !entry.file_type()?.is_dir() {
        return Err(Error::NotADirectory("definitions".to_string()));
    }

    let definitions_dir = fs::read_dir(entry.path())?;
    let files = definitions_dir
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|p| p == "xml"))
        .map(|e| e.path())
        .collect::<Vec<_>>();

    let mut protocols = List::with_capacity(files.len() as u32);
    for path in files {
        let content = fs::read_to_string(path)?;
        protocols.push(compile(pool, &content).unwrap());
    }

    Ok(protocols)
}
