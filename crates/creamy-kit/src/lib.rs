mod intern;
mod manifest;
mod parser;
mod protocol;
mod types;

use binrw::binrw;
use serde::{Deserialize, Serialize};

use crate::{intern::StringPool, manifest::Manifest};

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u16,
}

#[binrw]
#[brw(magic = b"CMY!", little)]
#[derive(Debug)]
pub struct BinaryPlugin {
    version: Version,
    manifest: Manifest,
    pool: StringPool,
}
