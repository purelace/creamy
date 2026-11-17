#![cfg_attr(coverage_nightly, coverage(off))]

use core::str::FromStr;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    #[default]
    Incoming,
    Outgoing,
    Duplex,
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
