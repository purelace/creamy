mod manifest;
mod utils;

use binrw::binrw;
use compiler_utils::{List, strpool::StringPool};
use serde::{Deserialize, Serialize};

use crate::manifest::{Manifest, Protocol};

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
    declare: List<Protocol>,
}
