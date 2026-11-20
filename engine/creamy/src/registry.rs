use alloc::{boxed::Box, string::String};
use core::{fmt::Debug, num::NonZeroU8};

use creamy_engine_core::scanner::Access;
use creamy_sdk::{ProtocolId, SubscriberId};
use hashbrown::HashMap;
use idmint::StackMint;
use rustc_hash::FxBuildHasher;

use crate::{GlobalProtocolId, engine::SYSTEM_PROTOCOL, scanner::CheckedProtocolApi};

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
        ids: Box<[ProtocolId; 256]>,
    },
    Private {
        consumer: Option<SubscriberId>,
        protocol_id: ProtocolId,
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
            protocol_id: SYSTEM_PROTOCOL,
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
        ids: &'a [ProtocolId; 256],
    },
    Private {
        is_shown: bool,
        consumer: Option<SubscriberId>,
        protocol_id: ProtocolId,
    },
}

impl Iterator for ConsumerListRepr<'_> {
    type Item = (SubscriberId, ProtocolId);

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
    type Item = (SubscriberId, ProtocolId);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Owner {
    id: SubscriberId,
    protocol_id: ProtocolId,
}

impl Owner {
    pub const fn new(plugin: SubscriberId, protocol_id: ProtocolId) -> Self {
        Self {
            id: plugin,
            protocol_id,
        }
    }

    pub const fn provider_id(self) -> SubscriberId {
        self.id
    }

    pub const fn protocol_id(self) -> ProtocolId {
        self.protocol_id
    }
}

pub struct ProtocolContext {
    protocol: CheckedProtocolApi,
    access: AccessPolicy,
    owner: Option<Owner>,
}

impl ProtocolContext {
    pub fn consumer_list(&self) -> ConsumerList<'_> {
        self.access.iter()
    }

    fn new(protocol: CheckedProtocolApi) -> Self {
        let policy = match protocol.access {
            Access::Public => AccessPolicy::Public {
                consumers: Bitset256::default(),
                ids: Box::from([SYSTEM_PROTOCOL; 256]),
            },
            Access::Private => AccessPolicy::Private {
                consumer: None,
                protocol_id: SYSTEM_PROTOCOL,
            },
        };

        Self {
            protocol,
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

    pub fn try_add_consumer(&mut self, consumer_id: SubscriberId, protocol_id: ProtocolId) -> bool {
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
                ids[consumer_id.as_usize()] = SYSTEM_PROTOCOL;
            }
            AccessPolicy::Private {
                consumer,
                protocol_id: pid,
            } => {
                debug_assert_eq!(*consumer, Some(consumer_id));
                *consumer = None;
                *pid = SYSTEM_PROTOCOL;
            }
        }
    }

    pub(crate) fn set_protocol_owner(&mut self, owner: Owner) {
        debug_assert!(self.owner.is_none());
        self.owner = Some(owner);
    }

    pub const fn protocol(&self) -> &CheckedProtocolApi {
        &self.protocol
    }
}

const MAX_PROTOCOLS: usize = 255;

pub struct ProtocolRegistry {
    map: HashMap<GlobalProtocolId, ProtocolContext, FxBuildHasher>,
    name_to_id: HashMap<String, GlobalProtocolId, FxBuildHasher>,
    global_id: StackMint,
}

impl Default for ProtocolRegistry {
    fn default() -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(MAX_PROTOCOLS, FxBuildHasher),
            name_to_id: HashMap::with_capacity_and_hasher(MAX_PROTOCOLS, FxBuildHasher),
            global_id: StackMint::new(1),
        }
    }
}

impl ProtocolRegistry {
    //pub(crate) fn declare_protocol(
    //    &mut self,
    //    api: CheckedProtocolApi,
    //) -> Option<&mut ProtocolContext> {
    //    let id = self.global_id.issue().expect("Too many protocols declared");
    //    let global_id = ProtocolId::new_u8(id).expect("zero value is reserved");
    //    self.map
    //        .try_insert(global_id, ProtocolContext::new(api))
    //        .map_err(|_| {})
    //}

    pub(crate) fn get_or_declare_protocol(
        &mut self,
        api: CheckedProtocolApi,
    ) -> &mut ProtocolContext {
        let id = self.global_id.issue().expect("Too many protocols declared");
        let global_id = GlobalProtocolId(ProtocolId::new_u8(id).expect("zero value is reserved"));
        self.name_to_id.insert(api.name.clone(), global_id);
        self.map
            .entry(global_id)
            .or_insert(ProtocolContext::new(api))
    }

    //pub(crate) fn try_get_protocol_context(&self, name: StringId) -> Option<&ProtocolContext> {
    //    self.map.get(&name)
    //}

    pub(crate) fn find_protocol(&self, protocol: &CheckedProtocolApi) -> Option<&ProtocolContext> {
        self.name_to_id
            .get(&protocol.name)
            .and_then(|id| self.map.get(id))
    }

    pub fn try_get_protocol_context_by_str(&self, name: &str) -> Option<&ProtocolContext> {
        self.name_to_id.get(name).and_then(|id| self.map.get(id))
    }

    pub(crate) fn get_mut_protocol_context(
        &mut self,
        id: GlobalProtocolId,
    ) -> &mut ProtocolContext {
        self.map
            .get_mut(&id)
            .expect("Context for protocol not found")
    }

    pub(crate) fn undeclare_protocol(&mut self, protocol: GlobalProtocolId) -> ProtocolContext {
        debug_assert!(self.map.contains_key(&protocol));
        let context = self.map.remove(&protocol).expect("Unreachable!");
        self.name_to_id.remove(&context.protocol.name);
        context
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
