#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]

mod error;
mod load;
mod write;

use std::{collections::HashMap, ffi::OsString, fs::DirEntry, path::Path, str::FromStr};

use binrw::binrw;
use creamy_manifest::{Arguments, Manifest};
use creamy_utils::{collections::List, strpool::StringPool, version::Version};
use creamy_xmlc::{ProtocolDefinition, compile};

use crate::error::DevKitError;

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
    pub fn name(&self) -> &str {
        self.manifest.name()
    }
}

pub fn compile_to_binary(plugin_dir: impl AsRef<Path>) -> Result<BinaryPlugin, DevKitError> {
    let plugin_dir = plugin_dir.as_ref();
    let files = std::fs::read_dir(plugin_dir)?
        .flatten()
        .map(|dir| (dir.file_name(), dir))
        .collect::<HashMap<_, _>>();

    let manifest_file = files
        .get(&OsString::from("manifest.toml"))
        .ok_or(DevKitError::MissingManifest)?;
    let (arguments, manifest) = compile_manifest(manifest_file)?;

    let core = read_core(plugin_dir, &arguments)?;

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
        core,
    })
}

fn compile_manifest(entry: &DirEntry) -> Result<(Arguments, Manifest), DevKitError> {
    if !entry.file_type()?.is_file() {
        return Err(DevKitError::NotAFile("manifest.toml".to_string()));
    }
    let manifest_content = std::fs::read_to_string(entry.path())?;

    Ok(Manifest::read_manifest(&manifest_content)?)
}

fn compile_protocols(
    entry: &DirEntry,
    pool: &mut StringPool,
) -> Result<List<ProtocolDefinition>, DevKitError> {
    if !entry.file_type()?.is_dir() {
        return Err(DevKitError::NotADirectory("definitions".to_string()));
    }

    let definitions_dir = std::fs::read_dir(entry.path())?;
    let files = definitions_dir
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|p| p == "xml"))
        .map(|e| e.path())
        .collect::<Vec<_>>();

    let mut protocols = List::with_capacity(files.len() as u32);
    for path in files {
        let content = std::fs::read_to_string(path)?;
        protocols.push(compile(pool, &content).unwrap());
    }

    Ok(protocols)
}

fn read_core(source: impl AsRef<Path>, arguments: &Arguments) -> Result<List<u8>, DevKitError> {
    let path = source.as_ref().join(&arguments.core);
    if path.is_file() {
        let mut header = arguments.runtime.as_bytes().to_vec();
        let data = std::fs::read(path)?;
        header.extend_from_slice(&data);
        Ok(List::wrap(header))
    } else {
        unimplemented!()
    }
}
