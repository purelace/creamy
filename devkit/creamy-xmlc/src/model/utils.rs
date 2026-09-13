use core::{fmt::Display, str::FromStr};

use crate::error::Fallback;

#[binrw::binrw]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Access {
    #[brw(magic(0u8))]
    /// Один поставщик - много пользователей
    Public,

    #[default]
    #[brw(magic(1u8))]
    /// Один поставщик - один пользователей
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

//#[cfg_attr(coverage_nightly, coverage(off))]
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

#[binrw::binrw]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    #[default]
    #[brw(magic = 0u8)]
    Incoming,
    #[brw(magic = 1u8)]
    Outgoing,
    #[brw(magic = 2u8)]
    Duplex,
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Incoming" => Ok(Direction::Incoming),
            "Outgoing" => Ok(Direction::Outgoing),
            "Duplex" => Ok(Direction::Duplex),
            _ => Err(()),
        }
    }
}
