#![cfg_attr(coverage_nightly, coverage(off))]

use core::{fmt::Display, str::FromStr};

use crate::error::Fallback;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Access {
    /// Single provider - many consumers
    Public,

    #[default]
    /// Single provider - single consumer
    Private,
}

impl FromStr for Access {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Public" => Ok(Access::Public),
            "Private" => Ok(Access::Private),
            _ => Err(()),
        }
    }
}

impl Display for Access {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let str = match self {
            Access::Public => "Public",
            Access::Private => "Private",
        };
        write!(f, "{str}")
    }
}

impl Fallback for Access {
    fn fallback() -> Self {
        Self::default()
    }
}
