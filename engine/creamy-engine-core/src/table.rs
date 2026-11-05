use core::num::NonZeroU8;

use creamy_phf::get_hash;

pub struct GroupTable<'a> {
    table: &'a [u8],
    table_size: u8,
    special: &'a [u64],
    special_size: u8,
}

impl<'a> GroupTable<'a> {
    #[must_use]
    pub const fn new(
        table: &'a [u8],
        table_size: u8,
        special: &'a [u64],
        special_size: u8,
    ) -> Self {
        Self {
            table,
            table_size,
            special,
            special_size,
        }
    }

    #[must_use]
    pub fn get_hash(&self, path: &str) -> u64 {
        get_hash(0xdead_beef, path.as_bytes())
    }

    pub fn get_group_id(&self, path: &str) -> NonZeroU8 {
        let group = self.get_hash(path) % u64::from(self.special_size);
        let perfect_seed = self.special[group as usize];
        let hash = get_hash(perfect_seed, path.as_bytes());
        let id = self.table[(hash % u64::from(self.table_size)) as usize];
        NonZeroU8::new(id).unwrap()
    }
}
