use std::{
    any::Any,
    io::{Read, Seek, Write},
};

use binrw::{BinRead, BinResult, BinWrite};
use strum::IntoEnumIterator;

use super::symbols::{
    BitsetValueSymbol, FieldSymbol, GroupSymbol, MessageSymbolType, OptionSymbol,
    StreamPayloadFieldSymbol, VariantSymbol,
};
use crate::utils::{BoundedVec, Range, VectorElement};

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, BinRead, BinWrite, strum::EnumIter,
)]
pub enum SymbolKey {
    #[brw(magic = 0u8)]
    Group,
    #[brw(magic = 1u8)]
    Message,
    #[brw(magic = 2u8)]
    BitsetValue,
    #[brw(magic = 3u8)]
    Option,
    #[brw(magic = 4u8)]
    Variant,
    #[brw(magic = 5u8)]
    Field,
    #[brw(magic = 6u8)]
    StreamPayloadField,
}

fn create_storage() -> Vec<Box<dyn UntypedStorage>> {
    let mut vec = vec![];
    for key in SymbolKey::iter() {
        let storage = match key {
            SymbolKey::Group => TypedStorage::<GroupSymbol>::boxed(),
            SymbolKey::Message => TypedStorage::<MessageSymbolType>::boxed(),
            SymbolKey::BitsetValue => TypedStorage::<BitsetValueSymbol>::boxed(),
            SymbolKey::Option => TypedStorage::<OptionSymbol>::boxed(),
            SymbolKey::Variant => TypedStorage::<VariantSymbol>::boxed(),
            SymbolKey::Field => TypedStorage::<FieldSymbol>::boxed(),
            SymbolKey::StreamPayloadField => TypedStorage::<StreamPayloadFieldSymbol>::boxed(),
        };

        vec.push(storage);
    }
    vec
}

fn read_storage<R: Read + Seek>(
    key: SymbolKey,
    reader: &mut R,
    endian: binrw::Endian,
) -> BinResult<Box<dyn UntypedStorage>> {
    let storage = match key {
        SymbolKey::Group => TypedStorage::<GroupSymbol>::read_boxed(reader, endian)?,
        SymbolKey::Message => TypedStorage::<MessageSymbolType>::read_boxed(reader, endian)?,
        SymbolKey::BitsetValue => TypedStorage::<BitsetValueSymbol>::read_boxed(reader, endian)?,
        SymbolKey::Option => TypedStorage::<OptionSymbol>::read_boxed(reader, endian)?,
        SymbolKey::Variant => TypedStorage::<VariantSymbol>::read_boxed(reader, endian)?,
        SymbolKey::Field => TypedStorage::<FieldSymbol>::read_boxed(reader, endian)?,
        SymbolKey::StreamPayloadField => {
            TypedStorage::<StreamPayloadFieldSymbol>::read_boxed(reader, endian)?
        }
    };

    Ok(storage)
}

pub trait Symbol:
    for<'a> BinRead<Args<'a> = ()>
    + for<'a> BinWrite<Args<'a> = ()>
    + PartialEq
    + Eq
    + std::fmt::Debug
    + VectorElement
    + Send
    + Sync
    + 'static
{
    const KEY: SymbolKey;
}

pub trait UntypedStorage: Send + Sync + std::fmt::Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn key(&self) -> SymbolKey;
    fn count(&self) -> u32;
    fn eq_dyn(&self, rhs: &dyn UntypedStorage) -> bool;

    fn bin_write_dyn(
        &self,
        writer: &mut dyn WriteSeek,
        endian: binrw::Endian,
    ) -> binrw::prelude::BinResult<()>;
}

impl BinWrite for dyn UntypedStorage {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        (): Self::Args<'_>,
    ) -> binrw::prelude::BinResult<()> {
        self.bin_write_dyn(writer, endian)
    }
}

impl PartialEq for dyn UntypedStorage {
    fn eq(&self, other: &Self) -> bool {
        self.eq_dyn(other)
    }
}

impl Eq for dyn UntypedStorage {}

pub trait WriteSeek: Write + Seek {}
impl<T: Write + Seek + ?Sized> WriteSeek for T {}

#[derive(Debug, PartialEq, Eq, BinRead, BinWrite)]
struct TypedStorage<S: Symbol> {
    inner: BoundedVec<S>,
}

impl<S: Symbol> TypedStorage<S> {
    fn boxed() -> Box<dyn UntypedStorage> {
        Box::new(TypedStorage::<S>::default())
    }

    fn read_boxed<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
    ) -> BinResult<Box<dyn UntypedStorage>> {
        Ok(Box::new(TypedStorage::<S>::read_options(
            reader,
            endian,
            (),
        )?))
    }
}

impl<S: Symbol> Default for TypedStorage<S> {
    fn default() -> Self {
        Self {
            inner: BoundedVec::default(),
        }
    }
}

impl<S: Symbol> UntypedStorage for TypedStorage<S> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn key(&self) -> SymbolKey {
        S::KEY
    }

    fn count(&self) -> u32 {
        self.inner.len()
    }

    fn eq_dyn(&self, rhs: &dyn UntypedStorage) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<TypedStorage<S>>() {
            self == rhs
        } else {
            false
        }
    }

    fn bin_write_dyn(
        &self,
        writer: &mut dyn WriteSeek,
        endian: binrw::Endian,
    ) -> binrw::prelude::BinResult<()> {
        let mut writer = std::io::BufWriter::new(writer);
        self.write_options(&mut writer, endian, ())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SymbolStorage {
    inner: Vec<Box<dyn UntypedStorage>>,
}

impl Default for SymbolStorage {
    fn default() -> Self {
        Self {
            inner: create_storage(),
        }
    }
}

impl BinRead for SymbolStorage {
    type Args<'a> = ();

    fn read_options<R: std::io::prelude::Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let mut map = create_storage();
        let length = u32::read_options(reader, endian, args)?;
        assert!(length as usize == map.len());

        for _ in 0..length {
            let key = SymbolKey::read_options(reader, endian, args)?;
            let index = key as usize;
            map[index] = read_storage(key, reader, endian)?;
        }

        Ok(Self { inner: map })
    }
}

impl BinWrite for SymbolStorage {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<()> {
        let length = self.inner.len() as u32;
        length.write_options(writer, endian, args)?;
        for storage in &self.inner {
            let key = storage.key();
            key.write_options(writer, endian, args)?;
            storage.write_options(writer, endian, args)?;
        }
        Ok(())
    }
}

impl SymbolStorage {
    pub fn add_symbol<S: Symbol>(&mut self, symbol: S) -> bool {
        let storage = &mut self.inner[S::KEY as usize];
        let Some(storage) = storage.as_any_mut().downcast_mut::<TypedStorage<S>>() else {
            unreachable!();
        };

        storage.inner.push(symbol)
    }

    #[must_use]
    pub fn get_symbol_slice<S: Symbol>(&self) -> &[S] {
        let storage = &self.inner[S::KEY as usize];
        let Some(storage) = storage.as_any().downcast_ref::<TypedStorage<S>>() else {
            unreachable!();
        };

        storage.inner.as_slice()
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn get_symbol_range<S: Symbol>(&self, range: S::RangeType) -> &[S] {
        &self.get_symbol_slice::<S>()[range.as_range()]
    }

    pub fn get_symbol_slice_mut<S: Symbol>(&mut self) -> &mut [S] {
        let storage = &mut self.inner[S::KEY as usize];
        let Some(storage) = storage.as_any_mut().downcast_mut::<TypedStorage<S>>() else {
            unreachable!();
        };

        &mut storage.inner
    }

    #[must_use]
    pub fn len_of<S: Symbol>(&self) -> usize {
        self.get_symbol_slice::<S>().len()
    }
}
