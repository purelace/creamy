use alloc::string::String;

use creamy_binary::{BinRead, BinWrite};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(
    BinRead,
    BinWrite,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct StringId(u16);

impl StringId {
    #[must_use]
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn value(&self) -> u16 {
        self.0
    }
}

#[derive(BinRead, BinWrite, Default, Debug, PartialEq, Eq)]
pub struct StringPool {
    map: HashMap<SmolStr, StringId>,
}

impl StringPool {
    /// Returns the [`StringId`] of the specified string
    ///
    /// # Panics
    ///
    /// Will panic if the string is missing in the [`StringPool`]
    ///
    /// # Examples
    ///
    /// ```
    /// use creamy_utils::strpool::{StringPool, StringId};
    ///
    /// let mut pool = StringPool::default();
    ///
    /// let foo_id0 = pool.get_id_or_add("Foo");
    /// let foo_id1 = pool.get_id("Foo");
    ///
    /// assert_eq!(foo_id0, foo_id1);
    /// ```
    ///
    /// # Panic example
    ///
    /// ```should_panic
    /// use creamy_utils::strpool::StringPool;
    ///
    /// let mut pool = StringPool::default();
    /// pool.get_id("Foo");
    /// ```
    #[must_use]
    pub fn get_id(&self, string: &str) -> StringId {
        *self
            .map
            .get(string)
            .expect("string is missing in the string pool")
    }

    #[must_use]
    pub fn try_get_id(&self, string: &str) -> Option<StringId> {
        self.map.get(string).copied()
    }

    /// Checks if the given string already exists in the pool and returns its [`StringId`].
    /// If the string is missing, inserts it into the pool and returns a newly generated [`StringId`].
    ///
    /// # Panics
    ///
    /// Will panic if the pool reaches its maximum capacity  (`u16::MAX`)
    ///
    /// # Examples
    /// ```
    /// use creamy_utils::strpool::StringPool;
    ///
    /// let mut pool = StringPool::default();
    /// assert!(pool.try_get_id("Foo").is_none());
    ///
    /// let _ = pool.get_id_or_add("Foo");
    /// assert!(pool.try_get_id("Foo").is_some());
    /// ```
    ///
    /// # Panic example
    /// ```should_panic
    /// use creamy_utils::strpool::{StringPool, StringId};
    ///
    /// let mut pool = StringPool::default();
    /// for i in 0..=u16::MAX {
    ///     pool.get_id_or_add(&format!("value{i}"));
    /// }
    /// pool.get_id_or_add(&format!("overflow"));
    /// ```
    pub fn get_id_or_add(&mut self, string: &str) -> StringId {
        self.map.get(string).copied().unwrap_or_else(|| {
            let value = u16::try_from(self.map.len()).expect("Error: string pool overflow");
            let id = StringId(value);
            self.map.insert(SmolStr::new(string), id);
            id
        })
    }

    /// Returns the string associated with [`StringId`]
    ///
    /// # Panics
    ///
    /// Will panic if the string is not found in the [`StringPool`]
    ///
    /// # Examples
    ///
    /// ```
    /// use creamy_utils::strpool::{StringPool, StringId};
    ///
    /// let mut pool = StringPool::default();
    ///
    /// let foo_id = pool.get_id_or_add("Foo");
    /// let foo_string = pool.get_string(foo_id);
    ///
    /// assert_eq!(foo_string, "Foo");
    /// ```
    ///
    /// # Panic example
    ///
    /// ```should_panic
    /// use creamy_utils::strpool::{StringPool, StringId};
    ///
    /// let mut pool = StringPool::default();
    /// pool.get_string(StringId::new(10));
    /// ```
    #[must_use]
    pub fn get_string(&self, id: StringId) -> &str {
        self.map
            .iter()
            .find(|(_, v)| **v == id)
            .expect("string not found")
            .0
    }
}

pub trait StringPoolResolver {
    fn resolve<'a>(&self, pool: &'a StringPool) -> &'a str;
}

pub trait StringPoolIntern {
    fn intern(&self, pool: &mut StringPool) -> StringId;
}

impl StringPoolIntern for String {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id_or_add(self)
    }
}

impl StringPoolIntern for &str {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        pool.get_id_or_add(self)
    }
}

impl StringPoolResolver for StringId {
    fn resolve<'a>(&self, pool: &'a StringPool) -> &'a str {
        pool.get_string(*self)
    }
}

impl StringPoolResolver for &StringId {
    fn resolve<'a>(&self, pool: &'a StringPool) -> &'a str {
        pool.get_string(**self)
    }
}
