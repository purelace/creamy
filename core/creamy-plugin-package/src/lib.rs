#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
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

use alloc::boxed::Box;

use ::semver::Version;
use creamy_binary::{BinRead, BinWrite};
use creamy_manifest::Manifest;

pub use crate::error::Error;

/// Represents a compiled binary plugin containing metadata, protocol definitions, and core logic.
#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
#[magic(b"CMY!")]
pub struct BinaryPlugin {
    /// The semantic version of the plugin.
    pub version: Version,
    /// The manifest containing plugin metadata.
    pub manifest: Manifest,
    /// The raw core logic of the plugin.
    core: Box<[u8]>,
}

impl BinaryPlugin {
    #[must_use]
    pub const fn core(&self) -> &[u8] {
        &self.core
    }
}
