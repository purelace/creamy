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
