#![cfg_attr(coverage_nightly, coverage(off))]

use core::str::FromStr;

use creamy_binary::BinWrite;

#[derive(BinWrite, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Direction {
    #[default]
    Incoming = 0,
    Outgoing = 1,
    Duplex = 2,
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "I" => Ok(Direction::Incoming),
            "O" => Ok(Direction::Outgoing),
            "D" => Ok(Direction::Duplex),
            _ => Err(()),
        }
    }
}
