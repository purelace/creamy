use alloc::{boxed::Box, string::String, vec, vec::Vec};
use core::mem::MaybeUninit;

use binrw::{BinRead, BinResult, BinWrite};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[binrw::binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[binrw::binrw]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct StringPool {
    #[br(parse_with = read_pool)]
    #[bw(write_with = write_pool)]
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
        self.map.get(string).copied().unwrap_or({
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
            .expect(&alloc::format!("string {id:#?} not found"))
            .0
    }
}

#[binrw::parser(reader: r, endian)]
fn read_pool() -> BinResult<HashMap<SmolStr, StringId>> {
    let len = u16::read_options(r, endian, ())? as usize;
    let mut map = HashMap::with_capacity(len);

    for id in 0..len {
        map.insert(read_str(r, endian, ())?, StringId(id as u16));
    }

    Ok(map)
}

#[binrw::writer(writer: w, endian)]
fn write_pool(pool: &HashMap<SmolStr, StringId>) -> BinResult<()> {
    let mut buffer: Vec<MaybeUninit<&SmolStr>> = vec![MaybeUninit::uninit(); pool.len()];
    for (string, id) in pool {
        buffer[id.0 as usize] = MaybeUninit::new(string);
    }

    (buffer.len() as u16).write_options(w, endian, ())?;
    for string in buffer {
        unsafe {
            write_str(string.assume_init_ref(), w, endian, ())?;
        }
    }

    Ok(())
}

#[binrw::parser(reader: r, endian)]
fn read_str() -> BinResult<SmolStr> {
    let len = u32::read_options(r, endian, ())?;
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf)?;
    let str = str::from_utf8(&buf).map_err(|e| binrw::Error::Custom {
        pos: r.stream_position().unwrap_or(0),
        err: Box::new(e),
    })?;

    Ok(SmolStr::new(str))
}

#[binrw::writer(writer: w, endian)]
fn write_str(string: &SmolStr) -> BinResult<()> {
    (string.len() as u32).write_options(w, endian, ())?;
    w.write_all(string.as_bytes())?;
    Ok(())
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
