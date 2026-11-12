#![allow(clippy::cast_possible_truncation)]
#![no_std]

extern crate alloc;

use alloc::{vec, vec::Vec};
use core::hash::{BuildHasher, Hasher};

#[must_use]
pub fn get_hash(seed: u64, value: &[u8]) -> u64 {
    let builder = wyhash::WyHasherBuilder::new(seed);
    let mut hasher = builder.build_hasher();
    core::hash::Hasher::write(&mut hasher, value);
    hasher.finish()
}

pub struct PerfectHashTable {
    pub table: Vec<u8>,
    pub special_table: Vec<u64>,
}

pub fn generate_perfect_hash_table<'a>(
    count_of_buckets: u8,
    size_of_table: u8,
    samples: impl Iterator<Item = (&'a str, u8)>,
) -> PerfectHashTable {
    let mut table = vec![0u8; size_of_table as usize];
    let mut special_table = vec![0u64; count_of_buckets as usize];

    let mut protocols: Vec<(usize, Vec<(&'a str, u8)>)> =
        core::iter::from_fn(|| Some((0, Vec::new())))
            .take(count_of_buckets as usize)
            .collect();

    for (path, protocol_id) in samples {
        let index = (get_hash(0xdead_beef, path.as_bytes()) % u64::from(count_of_buckets)) as usize;
        let (o, vec) = &mut protocols[index];
        vec.push((path, protocol_id));
        *o = index;
    }

    protocols.sort_by_key(|(_, v)| core::cmp::Reverse(v.len()));

    let mut temp = Vec::with_capacity(protocols[0].1.len());

    let mut i = 0;
    let mut hash: u64 = 1;
    while i < protocols.len() {
        if protocols[i].1.is_empty() {
            i += 1;
            continue;
        }

        let (original_index, bucket) = &protocols[i];

        for (path, protocol_id) in bucket {
            let index = get_hash(hash, path.as_bytes()) % u64::from(size_of_table);
            let index = index as usize;

            if table[index] == 0 {
                temp.push((index, *protocol_id));
                table[index] = *protocol_id;
            } else {
                hash += 1;
                for (index, _) in temp.drain(..) {
                    table[index] = 0;
                }
                break;
            }
        }

        if !temp.is_empty() {
            temp.clear();
            special_table[*original_index] = hash;
            i += 1;
        }
    }

    PerfectHashTable {
        table,
        special_table,
    }
}

#[cfg(test)]
mod tests {
    use crate::{generate_perfect_hash_table, get_hash};

    const SIZE: u8 = 16;
    const COUNT_OF_BUCKETS: u8 = 4;
    const PROTOCOLS: [(&str, u8); SIZE as usize] = [
        ("location.world0", 1),
        ("location.lobby", 2),
        ("system.core", 3),
        ("network.session", 4),
        ("player.data", 5),
        ("physics.layer", 6),
        ("render.pipeline", 7),
        ("audio.manager", 8),
        ("ai.controller", 9),
        ("ui.canvas", 10),
        ("database.cache", 11),
        ("security.auth", 12),
        ("metrics.collector", 13),
        ("event.bus", 14),
        ("resource.loader", 15),
        ("config.validator", 16),
    ];

    #[test]
    fn check_hash_table() {
        let pht = generate_perfect_hash_table(COUNT_OF_BUCKETS, SIZE, PROTOCOLS.iter().copied());

        for (path, id) in PROTOCOLS {
            let protocol = get_hash(0xdead_beef, path.as_bytes()) % u64::from(COUNT_OF_BUCKETS);
            let perfect_seed = pht.special_table[protocol as usize];
            let hash = get_hash(perfect_seed, path.as_bytes());
            assert_eq!(pht.table[(hash % u64::from(SIZE)) as usize], id);
        }
    }
}
