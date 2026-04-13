use std::collections::HashMap;

use binrw::{BinRead, BinResult, BinWrite, binrw};
use serde::{Deserialize, Serialize};

use crate::manifest::{read_bstr, write_bstr};

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
        self.map
            .get(string)
            .copied()
            .unwrap_or(StringId(self.map.len() as u32))
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
