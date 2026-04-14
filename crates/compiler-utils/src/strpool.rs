use std::collections::HashMap;

use binrw::{BinRead, BinResult, BinWrite, binrw};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StringId(u32);

#[binrw]
#[brw(little)]
#[derive(Debug, Default)]
pub struct StringPool {
    #[br(parse_with = read_pool)]
    #[bw(write_with = write_pool)]
    map: HashMap<String, StringId>,
}

impl StringPool {
    pub fn get_id(&mut self, string: &str) -> StringId {
        self.map.get(string).copied().unwrap_or_else(|| {
            let id = StringId(self.map.len() as u32);
            self.map.insert(string.to_string(), id);
            id
        })
    }
}

#[binrw::parser(reader: r, endian)]
fn read_pool() -> BinResult<HashMap<String, StringId>> {
    let len = u32::read_options(r, endian, ())? as usize;
    let mut map = HashMap::with_capacity(len);

    for id in 0..len {
        map.insert(read_bstr(r, endian, ())?, StringId(id as u32));
    }

    Ok(map)
}

#[binrw::writer(writer: w, endian)]
fn write_pool(pool: &HashMap<String, StringId>) -> BinResult<()> {
    let mut buffer = Vec::with_capacity(pool.len());
    for (string, id) in pool {
        buffer[id.0 as usize] = string;
    }

    (buffer.len() as u32).write_options(w, endian, ())?;
    for string in buffer {
        write_bstr(string, w, endian, ())?;
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
