#![cfg_attr(not(feature = "compile"), no_std)]

extern crate alloc;

mod error;

#[cfg(feature = "compile")]
pub mod compilation;

pub mod manifest {
    pub use creamy_manifest::*;
}

pub mod version {
    pub use semver::*;
}

pub mod pool {
    pub use creamy_utils::strpool::*;
}

pub mod model {
    pub use creamy_protocol_model::*;
}

use alloc::{boxed::Box, vec::Vec};

use ::semver::Version;
use creamy_binary::{BinRead, BinWrite};
use creamy_manifest::Manifest;
use creamy_protocol_model::definition::ProtocolModel;
use creamy_utils::strpool::StringPool;

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
    core: Box<[u8]>,
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
        &self.core
    }
}
