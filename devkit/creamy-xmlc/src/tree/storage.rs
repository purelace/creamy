use std::{any::Any, collections::HashMap};

use crate::utils::{BoundedVec, Range, VectorElement};

// 1. Сам узел должен возвращать свой TypeId (или наследовать std::any::Any)
pub trait Node: std::fmt::Debug + VectorElement + 'static {
    const KEY: NodeKey;
    const IS_TYPE: bool;
}

// 2. dyn-совместимый трейт (без генериков в методах!)
pub trait UntypedStorage: std::fmt::Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn is_type(&self) -> bool;
    fn count(&self) -> u32;
}

#[derive(Debug)]
struct TypedStorage<T: Node> {
    inner: BoundedVec<T>,
}

impl<T: Node> Default for TypedStorage<T> {
    fn default() -> Self {
        Self {
            inner: BoundedVec::default(),
        }
    }
}

impl<T: Node> UntypedStorage for TypedStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn is_type(&self) -> bool {
        T::IS_TYPE
    }

    fn count(&self) -> u32 {
        self.inner.len()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeKey(smol_str::SmolStr);

impl NodeKey {
    pub const fn new(value: &'static str) -> Self {
        Self(smol_str::SmolStr::new_static(value))
    }
}

impl std::fmt::Display for NodeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Default, Debug)]
pub struct NodeStorage {
    inner: HashMap<NodeKey, Box<dyn UntypedStorage>>,
}

impl NodeStorage {
    pub fn register_node<N: Node>(&mut self) {
        debug_assert!(!self.inner.contains_key(&N::KEY));

        self.inner
            .entry(N::KEY)
            .or_insert(Box::new(TypedStorage::<N>::default()));
    }

    pub fn add_node<N: Node>(&mut self, node: N) -> bool {
        let Some(storage) = self.inner.get_mut(&N::KEY) else {
            unreachable!("Node `{}` is not register", N::KEY);
        };

        let Some(storage) = storage.as_any_mut().downcast_mut::<TypedStorage<N>>() else {
            unreachable!();
        };

        storage.inner.push(node)
    }

    pub fn get_node_slice<N: Node>(&self) -> &[N] {
        let Some(storage) = self.inner.get(&N::KEY) else {
            unreachable!("Node `{}` is not register", N::KEY);
        };

        let Some(storage) = storage.as_any().downcast_ref::<TypedStorage<N>>() else {
            unreachable!();
        };

        storage.inner.as_slice()
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn get_node_range<N: Node>(&self, range: N::RangeType) -> &[N] {
        &self.get_node_slice()[range.as_range()]
    }

    //TODO: custom length
    pub fn len_of<N: Node>(&self) -> usize {
        self.get_node_slice::<N>().len()
    }

    // TODO: use `u16` instead
    pub fn type_count(&self) -> u32 {
        self.inner
            .values()
            .filter(|storage| storage.is_type())
            .map(|storage| storage.count())
            .sum()
    }
}
