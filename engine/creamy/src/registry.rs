use alloc::boxed::Box;
use core::{fmt::Debug, num::NonZeroU8};

use creamy_engine_core::package::model::{
    Access,
    definition::ProtocolModel,
    strpool::{StringId, StringPool},
};
use creamy_sdk::SubscriberId;
use hashbrown::HashMap;
use rustc_hash::FxBuildHasher;

#[derive(Default, Clone)]
struct Bitset256([u128; 2]);
impl Bitset256 {
    const BYTES: usize = 256 / 8;

    const fn as_byte_array_mut(&mut self) -> &mut [u8; Self::BYTES] {
        unsafe { core::mem::transmute(&mut self.0) }
    }

    const fn as_byte_array(&self) -> &[u8; Self::BYTES] {
        unsafe { core::mem::transmute(&self.0) }
    }

    /// Взводит бит на указанной позиции в 1.
    pub const fn set(&mut self, position: u8) {
        let position = position as usize;

        let index_of_byte = position / 8;
        let index_of_bit = position % 8;

        let array = self.as_byte_array_mut();
        array[index_of_byte] |= 1 << index_of_bit;
    }

    /// Обнуляет бит на указанной позиции.
    pub const fn unset(&mut self, position: u8) {
        let position = position as usize;

        let index_of_byte = position / 8;
        let index_of_bit = position % 8;

        let array = self.as_byte_array_mut();
        array[index_of_byte] &= !(1 << index_of_bit);
    }

    /// Проверяет, взведен ли бит на указанной позиции.
    pub const fn get(&self, position: u8) -> bool {
        let position = position as usize;

        let index_of_byte = position / 8;
        let index_of_bit = position % 8;

        let array = self.as_byte_array();

        ((array[index_of_byte] >> index_of_bit) & 1) == 1
    }

    const fn iter(&self) -> Bitset256Iterator<'_> {
        Bitset256Iterator::new(self)
    }
}

impl Debug for Bitset256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

struct Bitset256Iterator<'a> {
    bitset: &'a Bitset256,
    byte_index: usize, // 0..32
    current_byte: u8,
}

impl<'a> Bitset256Iterator<'a> {
    const fn new(bitset: &'a Bitset256) -> Self {
        let bytes = bitset.as_byte_array();
        Self {
            bitset,
            byte_index: 0,
            current_byte: bytes[0],
        }
    }
}

impl Iterator for Bitset256Iterator<'_> {
    type Item = NonZeroU8;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.bitset.as_byte_array();

        while self.byte_index < Bitset256::BYTES {
            if self.current_byte != 0 {
                let bit_index = self.current_byte.trailing_zeros() as u8;
                let global_bit_number = (self.byte_index as u8) * 8 + bit_index;
                self.current_byte &= self.current_byte - 1;

                return Some(unsafe { NonZeroU8::new_unchecked(global_bit_number) });
            }

            self.byte_index += 1;
            if self.byte_index < Bitset256::BYTES {
                self.current_byte = bytes[self.byte_index];
            }
        }

        None
    }
}

#[derive(Clone)]
enum AccessPolicy {
    Public {
        consumers: Bitset256,
        ids: Box<[NonZeroU8; 256]>,
    },
    Private {
        consumer: Option<SubscriberId>,
        protocol_id: NonZeroU8,
    },
}

impl Debug for AccessPolicy {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Public { consumers, ids } => f
                .debug_struct("Public")
                .field("consumers", consumers)
                .field("ids", ids)
                .finish(),
            Self::Private {
                consumer,
                protocol_id: id,
            } => f
                .debug_struct("Private")
                .field("consumer", consumer)
                .field("protocol_id", id)
                .finish(),
        }
    }
}

impl Default for AccessPolicy {
    fn default() -> Self {
        Self::Private {
            consumer: None,
            protocol_id: NonZeroU8::new(1).unwrap(),
        }
    }
}

impl AccessPolicy {
    fn iter(&self) -> ConsumerList<'_> {
        match self {
            AccessPolicy::Public { consumers, ids } => ConsumerList(ConsumerListRepr::Public {
                iter: consumers.iter(),
                ids,
            }),
            AccessPolicy::Private {
                consumer,
                protocol_id,
            } => ConsumerList(ConsumerListRepr::Private {
                is_shown: false,
                consumer: *consumer,
                protocol_id: *protocol_id,
            }),
        }
    }
}

enum ConsumerListRepr<'a> {
    Public {
        iter: Bitset256Iterator<'a>,
        ids: &'a [NonZeroU8; 256],
    },
    Private {
        is_shown: bool,
        consumer: Option<SubscriberId>,
        protocol_id: NonZeroU8,
    },
}

impl Iterator for ConsumerListRepr<'_> {
    type Item = (SubscriberId, NonZeroU8);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ConsumerListRepr::Public { iter, ids } => {
                if let Some(id) = iter.next() {
                    return Some((SubscriberId::new(id), ids[id.get() as usize]));
                }
                None
            }
            ConsumerListRepr::Private {
                is_shown,
                consumer,
                protocol_id,
            } => {
                if *is_shown {
                    return None;
                }

                *is_shown = true;
                Some(((*consumer)?, *protocol_id))
            }
        }
    }
}

pub struct ConsumerList<'a>(ConsumerListRepr<'a>);
impl Iterator for ConsumerList<'_> {
    type Item = (SubscriberId, NonZeroU8);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Owner {
    id: SubscriberId,
    protocol_id: NonZeroU8,
}

impl Owner {
    pub const fn new(plugin: SubscriberId, protocol_id: NonZeroU8) -> Self {
        Self {
            id: plugin,
            protocol_id,
        }
    }

    pub const fn provider_id(self) -> SubscriberId {
        self.id
    }

    pub const fn protocol_id(self) -> NonZeroU8 {
        self.protocol_id
    }
}

pub struct ProtocolContext {
    model: ProtocolModel,
    access: AccessPolicy,
    owner: Option<Owner>,
}

impl ProtocolContext {
    pub fn consumer_list(&self) -> ConsumerList<'_> {
        self.access.iter()
    }

    fn new(model: ProtocolModel) -> Self {
        let policy = match model.access() {
            Access::Public => AccessPolicy::Public {
                consumers: Bitset256::default(),
                ids: Box::from([NonZeroU8::new(1).unwrap(); 256]),
            },
            Access::Private => AccessPolicy::Private {
                consumer: None,
                protocol_id: NonZeroU8::new(1).unwrap(),
            },
        };

        Self {
            model,
            access: policy,
            owner: None,
        }
    }

    pub(crate) fn get_owner(&self) -> Owner {
        self.try_get_owner().expect("Unreachable!")
    }

    pub(crate) fn try_get_owner(&self) -> Option<Owner> {
        self.owner
    }

    pub fn can_add_consumer(&self) -> bool {
        match &self.access {
            AccessPolicy::Public { .. } => true,
            AccessPolicy::Private { consumer, .. } => consumer.is_none(),
        }
    }

    pub fn try_add_consumer(&mut self, consumer_id: SubscriberId, protocol_id: NonZeroU8) -> bool {
        match &mut self.access {
            AccessPolicy::Public { consumers, ids } => {
                consumers.set(consumer_id.as_u8());
                ids[consumer_id.as_usize()] = protocol_id;
                true
            }
            AccessPolicy::Private {
                consumer,
                protocol_id: gid,
            } => {
                if consumer.is_some() {
                    false
                } else {
                    *consumer = Some(consumer_id);
                    *gid = protocol_id;
                    true
                }
            }
        }
    }

    pub fn remove_consumer(&mut self, consumer_id: SubscriberId) {
        match &mut self.access {
            AccessPolicy::Public { consumers, ids } => {
                debug_assert!(consumers.get(consumer_id.as_u8()));
                consumers.unset(consumer_id.as_u8());
                ids[consumer_id.as_usize()] = NonZeroU8::new(1).unwrap();
            }
            AccessPolicy::Private {
                consumer,
                protocol_id: gid,
            } => {
                debug_assert_eq!(*consumer, Some(consumer_id));
                *consumer = None;
                *gid = NonZeroU8::new(1).unwrap();
            }
        }
    }

    pub(crate) fn set_protocol_owner(&mut self, owner: Owner) {
        debug_assert!(self.owner.is_none());
        self.owner = Some(owner);
    }

    pub const fn model(&self) -> &ProtocolModel {
        &self.model
    }
}

pub struct ProtocolRegistry {
    pool: StringPool,
    map: HashMap<StringId, ProtocolContext, FxBuildHasher>,
}

impl Default for ProtocolRegistry {
    fn default() -> Self {
        Self {
            pool: StringPool::default(),
            map: HashMap::with_capacity_and_hasher(256, FxBuildHasher),
        }
    }
}

impl ProtocolRegistry {
    pub(crate) fn replace_strings(&mut self, pool: &StringPool, model: &mut ProtocolModel) {
        model.replace_identifiers(pool, &mut self.pool);
    }

    pub(crate) fn get_mut_protocol_context_by_name(
        &mut self,
        name: StringId,
    ) -> &mut ProtocolContext {
        self.map
            .get_mut(&name)
            .expect("Context for protocol model not found")
    }

    pub(crate) fn get_or_declare_protocol(
        &mut self,
        model: ProtocolModel,
    ) -> (&StringPool, &mut ProtocolContext) {
        let context = self
            .map
            .entry(model.name())
            .or_insert(ProtocolContext::new(model));
        (&self.pool, context)
    }

    pub(crate) fn try_get_protocol_context(&self, name: StringId) -> Option<&ProtocolContext> {
        self.map.get(&name)
    }

    pub fn try_get_protocol_context_by_str(&self, name: &str) -> Option<&ProtocolContext> {
        self.pool.try_get_id(name).and_then(|id| self.map.get(&id))
    }

    pub const fn pool(&self) -> &StringPool {
        &self.pool
    }

    pub(crate) fn undeclare_protocol(&mut self, protocol: StringId) -> ProtocolContext {
        debug_assert!(self.map.contains_key(&protocol));
        self.map.remove(&protocol).expect("Unreachable!")
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::Bitset256;

    #[test]
    fn bitset() {
        let mut bitset = Bitset256::default();
        bitset.set(1);
        bitset.set(5);
        bitset.set(8);
        bitset.set(18);

        assert!(bitset.get(8));
        bitset.unset(8);
        assert!(!bitset.get(8));

        let result = bitset.iter().collect::<Vec<_>>();
        assert_eq!(result[0].get(), 1);
        assert_eq!(result[1].get(), 5);
        assert_eq!(result[2].get(), 18);
    }
}
