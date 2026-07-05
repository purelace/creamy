#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StreamChunkType {
    Single = 0,
    Start = 1,
    Payload = 2,
    End = 3,
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamId(u8);
impl StreamId {
    pub const fn new(id: u8) -> Self {
        assert!(id <= 64);
        Self(id)
    }

    pub const fn new_trim(id: u8) -> Self {
        Self(id & 0b_0011_1111)
    }

    pub const fn safe_new(id: u8) -> Option<Self> {
        if id > 64 {
            return None;
        }

        Some(Self(id))
    }

    pub const fn value(&self) -> u8 {
        self.0
    }

    pub const fn is_broken(&self) -> bool {
        self.0 > 64
    }
}
