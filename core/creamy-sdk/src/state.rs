use core::{cell::UnsafeCell, num::NonZeroU8};

use crate::{
    SubscriberId,
    api::{CustomHandler, Plugin},
    generated::{
        dispatcher::{self, MessageHandler},
        system::builtin::{GroupDeclared, GroupUndeclared, StreamCancel, StreamKeepAlive},
    },
    message::UntypedMessage,
};

#[derive(Clone, Copy)]
struct Slice {
    offset: u8,
    length: u8,
}

impl Slice {
    const EMPTY: Self = Self {
        offset: 0,
        length: 0,
    };
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl core::fmt::Debug for Table {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let iter = self
            .slices
            .iter()
            .enumerate()
            .map(|(i, &Slice { offset, length })| {
                (
                    i,
                    &self.inner[offset as usize..offset as usize + length as usize],
                )
            });

        f.debug_map().entries(iter).finish()
    }
}

pub(crate) struct Table {
    inner: [Option<SubscriberId>; 256],
    slices: [Slice; 256],
    total: u8,
}

impl Table {
    const EMPTY: Self = Self {
        inner: [None; 256],
        slices: [Slice::EMPTY; 256],
        total: 0,
    };

    //TODO: represent dst and group as SubscriberId
    fn add(&mut self, group: u8, dst: u8) {
        assert!(self.total != u8::MAX, "Table is full");

        let group = group as usize;

        if self.slices[group].length == 0 {
            self.slices[group].offset = self.total;
        }

        let start = self.slices[group].offset;
        let len = self.slices[group].length;
        let insertion_point = start + len;

        if insertion_point < self.total {
            let insertion_point = insertion_point as usize;
            let src = insertion_point..self.total as usize;
            self.inner.copy_within(src, insertion_point + 1);
        }

        self.inner[insertion_point as usize] = Some(SubscriberId::new_u8(dst).unwrap());

        for s in &mut self.slices {
            if s.length > 0 && s.offset >= insertion_point {
                s.offset += 1;
            }
        }

        self.slices[group].length += 1;
        self.total += 1;
    }

    fn remove(&mut self, group: u8, dst: u8) {
        let group = group as usize;
        let mut s = self.slices[group];
        if s.length == 0 {
            return;
        }

        let start = s.offset as usize;
        let len = s.length as usize;

        let slice = &mut self.inner[start..start + len];
        let mut remove_index = None;
        for (i, entry) in slice.iter().enumerate() {
            if let Some(id) = entry
                && id.as_u8() == dst
            {
                remove_index = Some(i);
                break;
            }
        }

        let Some(index_in_slice) = remove_index else {
            return;
        };
        let global_remove_index = start + index_in_slice;

        if global_remove_index < (self.total as usize) - 1 {
            let src = (global_remove_index + 1)..(self.total as usize);
            self.inner.copy_within(src, global_remove_index);
        }

        self.inner[(self.total as usize) - 1] = None;

        for other_slice in &mut self.slices {
            if other_slice.length > 0 && other_slice.offset > global_remove_index as u8 {
                other_slice.offset -= 1;
            }
        }

        s.length -= 1;
        self.total -= 1;

        if s.length == 0 {
            s.offset = 0;
        }

        self.slices[group] = s;
    }

    pub fn get(&self, group: NonZeroU8) -> &[SubscriberId] {
        let Slice { offset, length } = self.slices[group.get() as usize];
        let offset = offset as usize;
        let length = length as usize;
        let slice = &self.inner[offset..offset + length];
        debug_assert!(slice.iter().all(Option::is_some));
        unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), slice.len()) }
    }
}

#[repr(transparent)]
struct SyncUnsafeCell<T>(UnsafeCell<T>);
unsafe impl<T: Sync> Sync for SyncUnsafeCell<T> {}

static DST_TABLE: SyncUnsafeCell<Table> = SyncUnsafeCell(UnsafeCell::new(Table::EMPTY));

#[must_use]
pub(crate) const fn get_dst_table() -> *const Table {
    DST_TABLE.0.get().cast_const()
}

pub struct InnerState<P: Plugin> {
    plugin: P,
}

impl<P: Plugin> InnerState<P> {
    pub const fn new(plugin: P) -> Self {
        Self { plugin }
    }

    pub fn notify(&mut self) {
        self.plugin.notify();
    }
}

impl<P: Plugin> CustomHandler for InnerState<P> {
    fn handle_message(&mut self, dispatch_value: u32, message: cbus_core::UntypedMessage) {
        dispatcher::dispatch_message(dispatch_value, message, self);
    }

    fn handle_on_group_enabled_event(&mut self, _: u8) {}

    fn handle_on_group_disabled_event(&mut self, _: u8) {}
}

impl<P: Plugin> MessageHandler for InnerState<P> {
    fn handle_group_declared(&mut self, message: GroupDeclared) {
        unsafe {
            (*DST_TABLE.0.get()).add(message.group_id, message.provider);
        }
        self.plugin.handle_on_group_enabled_event(message.group_id);
    }

    fn handle_group_undeclared(&mut self, message: GroupUndeclared) {
        unsafe {
            (*DST_TABLE.0.get()).remove(message.group_id, message.provider);
        }
        self.plugin.handle_on_group_disabled_event(message.group_id);
    }

    fn handle_stream_keep_alive(&mut self, message: StreamKeepAlive) {}

    fn handle_stream_cancel(&mut self, message: StreamCancel) {}

    fn handle_unknown_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
        self.plugin.handle_message(dispatch_value, message);
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};
    use core::num::NonZeroU8;

    use cbus_core::SubscriberId;

    use super::Table;

    #[test]
    fn test_basic_add_and_remove() {
        let mut table = Table::EMPTY;

        table.add(10, 42);
        table.add(10, 43);
        table.add(5, 100);

        assert_eq!(table.slices[5].length, 1);
        assert_eq!(table.slices[10].length, 2);
        assert_eq!(table.total, 3);

        assert_eq!(table.slices[5].offset, 2);
        assert_eq!(table.slices[10].offset, 0);

        table.remove(10, 42);

        assert_eq!(table.slices[10].length, 1);
        assert_eq!(table.total, 2);

        let g10_start = table.slices[10].offset as usize;
        assert_eq!(table.inner[g10_start].unwrap().as_u8(), 43);

        assert_eq!(
            table.get(NonZeroU8::new(5).unwrap()),
            [SubscriberId::new(NonZeroU8::new(100).unwrap())]
        );
    }

    #[test]
    fn test_shift_and_order_preservation() {
        let mut table = Table::EMPTY;

        table.add(1, 11);
        table.add(2, 22);
        table.add(3, 33);

        table.add(1, 12);

        assert_eq!(table.slices[1].offset, 0);
        assert_eq!(table.slices[1].length, 2);

        assert_eq!(table.slices[2].offset, 2);
        assert_eq!(table.slices[3].offset, 3);

        let res: Vec<u8> = table.inner[0..4]
            .iter()
            .map(|o| o.unwrap().as_u8())
            .collect();
        assert_eq!(res, vec![11, 12, 22, 33]);
    }

    #[test]
    fn test_remove_last_element_in_group() {
        let mut table = Table::EMPTY;

        table.add(5, 55);
        table.add(6, 66);

        table.remove(5, 55);

        assert_eq!(table.slices[5].length, 0);
        assert_eq!(table.slices[6].offset, 0);
        assert_eq!(table.inner[0].unwrap().as_u8(), 66);
    }

    #[test]
    #[should_panic(expected = "Table is full")]
    fn test_table_overflow() {
        let mut table = Table::EMPTY;

        for i in 0..=255u8 {
            let dst = i % 250;
            let dst = dst + u8::from(dst == 0);

            table.add(0, dst);
        }

        table.add(0, 255);
    }

    #[test]
    fn test_remove_non_existent() {
        let mut table = Table::EMPTY;
        table.add(1, 10);

        table.remove(1, 99);
        table.remove(5, 10);

        assert_eq!(table.total, 1);
        assert_eq!(table.slices[1].offset, 0);
        assert_eq!(table.slices[1].length, 1);
    }
}
