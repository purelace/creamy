use std::{collections::HashMap, mem::MaybeUninit, num::NonZeroU16};

use binrw::{BinRead, BinResult, BinWrite};
use serde::{Deserialize, Serialize};

#[derive(
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
    BinRead,
    BinWrite,
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

#[derive(
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
    BinRead,
    BinWrite,
)]
pub struct NonZeroStringId(NonZeroU16);

impl NonZeroStringId {
    #[must_use]
    pub const fn new(id: NonZeroU16) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn as_string_id(self) -> StringId {
        StringId(self.0.get())
    }
}

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
pub struct StringPool {
    #[br(parse_with = read_pool)]
    #[bw(write_with = write_pool)]
    map: HashMap<String, StringId>,
}

impl Default for StringPool {
    fn default() -> Self {
        let mut instance = Self {
            map: HashMap::default(),
        };

        //TODO: fix
        instance.get_id_or_add("u8");
        instance.get_id_or_add("u16");
        instance.get_id_or_add("u32");
        instance.get_id_or_add("u64");
        instance.get_id_or_add("u128");

        instance.get_id_or_add("i8");
        instance.get_id_or_add("i16");
        instance.get_id_or_add("i32");
        instance.get_id_or_add("i64");
        instance.get_id_or_add("i128");

        instance.get_id_or_add("f32");
        instance.get_id_or_add("f64");
        instance.get_id_or_add("bool");

        instance
    }
}

impl StringPool {
    pub fn remove(&mut self, id: StringId) -> String {
        let mut removed_iterator = self.map.extract_if(|_key, value| *value == id);

        if let Some((k, _)) = removed_iterator.next() {
            return k;
        }
        todo!();
    }

    #[must_use]
    pub fn get_id(&self, string: &str) -> StringId {
        *self.map.get(string).unwrap()
    }

    pub fn get_id_or_add(&mut self, string: &str) -> StringId {
        self.map.get(string).copied().unwrap_or_else(|| {
            let id = StringId(self.map.len() as u16);
            self.map.insert(string.to_string(), id);
            id
        })
    }

    pub fn get_non_zero_id(&mut self, string: &str) -> NonZeroStringId {
        self.map.get(string).copied().map_or_else(
            || {
                let raw_id = NonZeroU16::new(self.map.len() as u16).unwrap();
                self.map
                    .insert(string.to_string(), StringId(self.map.len() as u16));
                NonZeroStringId(raw_id)
            },
            |id| NonZeroStringId(NonZeroU16::new(id.0).unwrap()),
        )
    }

    #[must_use]
    pub fn get_string(&self, id: StringId) -> &str {
        self.map.iter().find(|(_, v)| **v == id).unwrap().0
    }

    pub fn remove_by_string(&mut self, string: &str) -> Option<StringId> {
        self.map.remove(string)
    }
}

#[binrw::parser(reader: r, endian)]
fn read_pool() -> BinResult<HashMap<String, StringId>> {
    let len = u16::read_options(r, endian, ())? as usize;
    let mut map = HashMap::with_capacity(len);

    for id in 0..len {
        map.insert(read_bstr(r, endian, ())?, StringId(id as u16));
    }

    Ok(map)
}

#[binrw::writer(writer: w, endian)]
fn write_pool(pool: &HashMap<String, StringId>) -> BinResult<()> {
    let mut buffer: Vec<MaybeUninit<&String>> = vec![MaybeUninit::uninit(); pool.len()];
    for (string, id) in pool {
        buffer[id.0 as usize] = MaybeUninit::new(string);
    }

    (buffer.len() as u16).write_options(w, endian, ())?;
    for string in buffer {
        unsafe {
            write_bstr(string.assume_init_ref(), w, endian, ())?;
        }
    }

    Ok(())
}

#[binrw::parser(reader: r, endian)]
fn read_bstr() -> BinResult<String> {
    let len = u32::read_options(r, endian, ())?;
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| binrw::Error::Custom {
        pos: r.stream_position().unwrap_or(0),
        err: Box::new(e),
    })
}

#[binrw::writer(writer: w, endian)]
fn write_bstr(string: &String) -> BinResult<()> {
    (string.len() as u32).write_options(w, endian, ())?;
    w.write_all(string.as_bytes())?;
    Ok(())
}
