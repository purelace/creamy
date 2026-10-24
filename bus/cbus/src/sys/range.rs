use cbus_core::SubscriberId;

/// Inclusive range of almost all valid subscriber ids
#[derive(Clone, Copy)]
pub(crate) struct SubscriberRange {
    start: SubscriberId,
    end: u8,
}

impl SubscriberRange {
    pub const fn new(start: SubscriberId, end: u8) -> Self {
        Self { start, end }
    }

    #[cfg(target_feature = "sse4.1")]
    pub const fn start(self) -> u8 {
        self.start.as_u8()
    }

    #[cfg(target_feature = "sse4.1")]
    pub const fn end(self) -> u8 {
        self.end
    }

    pub const fn contains(self, value: u8) -> bool {
        value >= self.start.as_u8() && value <= self.end
    }
}

impl IntoIterator for SubscriberRange {
    type Item = SubscriberId;

    type IntoIter = RangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        RangeIterator {
            end: self.end,
            current: Some(self.start),
        }
    }
}

pub(crate) struct RangeIterator {
    end: u8,
    current: Option<SubscriberId>,
}

impl Iterator for RangeIterator {
    type Item = SubscriberId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?.as_non_zero_u8();

        if current.get() > self.end {
            return None;
        }

        let result = SubscriberId::new(current);
        self.current = current.checked_add(1).map(SubscriberId::new);
        Some(result)
    }
}
