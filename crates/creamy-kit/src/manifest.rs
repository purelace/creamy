use std::num::NonZeroU8;

use binrw::{BinRead, BinResult, BinWrite, binrw};
use serde::{Deserialize, Serialize};

use crate::Version;

#[binrw::parser(reader: r, endian)]
pub fn read_bstr() -> BinResult<String> {
    let len = u32::read_options(r, endian, ())?;
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| binrw::Error::Custom {
        pos: r.stream_position().unwrap_or(0),
        err: Box::new(e),
    })
}

#[binrw::writer(writer: w, endian)]
pub fn write_bstr(string: &String) -> BinResult<()> {
    (string.len() as u32).write_options(w, endian, ())?;
    w.write_all(string.as_bytes())?;
    Ok(())
}

#[binrw::writer(writer: w, endian)]
fn write_vec_bstr(value: &Vec<String>) -> BinResult<()> {
    (value.len() as u32).write_options(w, endian, ())?;
    for s in value {
        write_bstr(s, w, endian, ())?;
    }
    Ok(())
}

#[binrw::parser(reader: r, endian)]
fn read_vec_bstr(len: u32) -> BinResult<Vec<String>> {
    let mut buf = vec![String::new(); len as usize];
    for _ in 0..len {
        buf.push(read_bstr(r, endian, ())?);
    }

    Ok(buf)
}

#[binrw::parser(reader: r, endian)]
fn read_bproto() -> BinResult<Protocol> {
    let name = read_bstr(r, endian, ())?;
    let group = Option::<NonZeroU8>::read_options(r, endian, ())?;
    Ok(Protocol { name, group })
}

#[binrw::writer(writer: w, endian)]
fn write_bproto(protocol: &Protocol) -> BinResult<()> {
    protocol.write_options(w, endian, ())
}

#[binrw::parser(reader: r, endian)]
fn read_vec_bproto(len: u32) -> BinResult<Vec<Protocol>> {
    let mut buf = vec![Protocol::default(); len as usize];
    for _ in 0..len {
        buf.push(read_bproto(r, endian, ())?);
    }
    Ok(buf)
}

#[binrw::writer(writer: w, endian)]
fn write_vec_bproto(value: &Vec<Protocol>) -> BinResult<()> {
    (value.len() as u32).write_options(w, endian, ())?;
    for s in value {
        write_bproto(s, w, endian, ())?;
    }
    Ok(())
}

#[binrw]
#[brw(little)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    #[br(parse_with = read_bstr)]
    #[bw(write_with = write_bstr)]
    id: String,

    #[br(parse_with = read_bstr)]
    #[bw(write_with = write_bstr)]
    name: String,

    version: Version,

    #[br(parse_with = read_bstr)]
    #[bw(write_with = write_bstr)]
    description: String,

    #[br(parse_with = read_bstr)]
    #[bw(write_with = write_bstr)]
    repo: String,

    #[br(temp)]
    #[bw(calc = authors.len() as u32)]
    authors_len: u32,

    #[br(parse_with = read_vec_bstr, args(authors_len))]
    #[bw(write_with = write_vec_bstr)]
    authors: Vec<String>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Protocol {
    #[br(parse_with = read_bstr)]
    #[bw(write_with = write_bstr)]
    name: String,
    group: Option<NonZeroU8>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    package: Package,

    #[br(temp)]
    #[bw(calc = protocols.len() as u32)]
    protocols_len: u32,

    #[br(parse_with = read_vec_bproto, args(protocols_len))]
    #[bw(write_with = write_vec_bproto)]
    protocols: Vec<Protocol>,
}
