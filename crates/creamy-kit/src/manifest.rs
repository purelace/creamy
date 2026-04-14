use std::num::NonZeroU8;

use binrw::binrw;
use compiler_utils::{BString, List};
use serde::{Deserialize, Serialize};

use crate::Version;

#[binrw]
#[brw(little)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    id: BString,
    name: BString,
    version: Version,
    description: BString,
    repo: BString,
    authors: List<BString>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Protocol {
    name: BString,
    group: Option<NonZeroU8>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    package: Package,
    protocols: List<Protocol>,
}
