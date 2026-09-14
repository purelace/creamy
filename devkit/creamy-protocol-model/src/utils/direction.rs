use core::str::FromStr;

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
