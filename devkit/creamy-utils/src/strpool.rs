use alloc::{boxed::Box, string::String, vec, vec::Vec};
use core::{mem::MaybeUninit, str::FromStr};

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
    #[must_use]
    pub fn get_id(&self, string: &str) -> StringId {
        *self.map.get(string).unwrap()
    }

    #[must_use]
    pub fn try_get_id(&self, string: &str) -> Option<StringId> {
        self.map.get(string).copied()
    }

    pub fn get_id_or_add(&mut self, string: &str) -> StringId {
        self.map.get(string).copied().unwrap_or_else(|| {
            let id = StringId(self.map.len() as u16);
            self.map.insert(SmolStr::new(string), id);
            id
        })
    }

    #[must_use]
    pub fn get_string(&self, id: StringId) -> &str {
        self.map.iter().find(|(_, v)| **v == id).unwrap().0
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

    SmolStr::from_str(str).map_err(|e| binrw::Error::Custom {
        pos: r.stream_position().unwrap_or(0),
        err: Box::new(e),
    })
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
