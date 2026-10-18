use alloc::{boxed::Box, vec::Vec};
use core::any::Any;

use creamy_binary::{
    BinRead, BinWrite, BinaryReader, BinaryWriter, DynBinaryWrite, DynWrite, DynWriter, Read,
    ReadError, WriteError,
};
use strum::{EnumIter, IntoEnumIterator};

use super::symbols::{
    BitsetValueSymbol, FieldSymbol, GroupSymbol, MessageSymbolType, OptionSymbol,
    StreamPayloadFieldSymbol, VariantSymbol,
};
use crate::utils::{BoundedVec, Range, VectorElement};

#[repr(u8)]
#[derive(BinRead, BinWrite, EnumIter, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolKey {
    Group = 0,
    Message = 1,
    BitsetValue = 2,
    Option = 3,
    Variant = 4,
    Field = 5,
    StreamPayloadField = 6,
}

fn create_storage() -> Vec<Box<dyn UntypedStorage>> {
    let mut vec = alloc::vec![];
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

fn read_storage<R: creamy_binary::Read>(
    key: SymbolKey,
    reader: &mut R,
) -> Result<Box<dyn UntypedStorage>, ReadError<R::Error>> {
    let storage = match key {
        SymbolKey::Group => TypedStorage::<GroupSymbol>::read_boxed(reader)?,
        SymbolKey::Message => TypedStorage::<MessageSymbolType>::read_boxed(reader)?,
        SymbolKey::BitsetValue => TypedStorage::<BitsetValueSymbol>::read_boxed(reader)?,
        SymbolKey::Option => TypedStorage::<OptionSymbol>::read_boxed(reader)?,
        SymbolKey::Variant => TypedStorage::<VariantSymbol>::read_boxed(reader)?,
        SymbolKey::Field => TypedStorage::<FieldSymbol>::read_boxed(reader)?,
        SymbolKey::StreamPayloadField => {
            TypedStorage::<StreamPayloadFieldSymbol>::read_boxed(reader)?
        }
    };

    Ok(storage)
}

pub trait Symbol:
    BinaryReader
    + BinaryWriter
    + PartialEq
    + Eq
    + core::fmt::Debug
    + VectorElement
    + Send
    + Sync
    + 'static
{
    const KEY: SymbolKey;
}

pub trait UntypedStorage: DynBinaryWrite + Send + Sync + core::fmt::Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn key(&self) -> SymbolKey;
    fn count(&self) -> u32;
    fn eq_dyn(&self, rhs: &dyn UntypedStorage) -> bool;
}

impl PartialEq for dyn UntypedStorage {
    fn eq(&self, other: &Self) -> bool {
        self.eq_dyn(other)
    }
}

impl Eq for dyn UntypedStorage {}

//pub trait WriteSeek: Write + Seek {}
//impl<T: Write + Seek + ?Sized> WriteSeek for T {}

#[derive(Debug, PartialEq, Eq, BinRead, BinWrite)]
struct TypedStorage<S: Symbol> {
    inner: BoundedVec<S>,
}

impl<S: Symbol> TypedStorage<S> {
    fn boxed() -> Box<dyn UntypedStorage> {
        Box::new(TypedStorage::<S>::default())
    }

    fn read_boxed<R: Read>(reader: &mut R) -> Result<Box<dyn UntypedStorage>, ReadError<R::Error>> {
        Ok(Box::new(TypedStorage::<S>::read_ne(reader)?))
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

    //fn bin_write_dyn(
    //    &self,
    //    writer: &mut dyn WriteSeek,
    //) -> Result<(), WriteError<<Writer<'_> as ErrorType>::Error>> {
    //    let mut writer = Writer { object: writer };
    //    let xd: Result<(), WriteError<<Writer<'_> as ErrorType>::Error>> = self.write(&mut writer);
    //}
}

/*
struct Writer<'a> {
    object: &'a mut dyn WriteSeek,
}

impl Write for Writer<'_> {
    fn write(&mut self, buf: &[u8]) -> binrw::io::Result<usize> {
        self.object.write(buf)
    }

    fn flush(&mut self) -> binrw::io::Result<()> {
        self.object.flush()
    }
}

impl Seek for Writer<'_> {
    fn seek(&mut self, pos: binrw::io::SeekFrom) -> binrw::io::Result<u64> {
        self.object.seek(pos)
    }
}
*/
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

impl BinaryReader for SymbolStorage {
    fn read<R: creamy_binary::Read, E: creamy_binary::Endianness>(
        reader: &mut R,
    ) -> Result<Self, creamy_binary::ReadError<R::Error>> {
        let mut map = create_storage();
        let length = u32::read::<R, E>(reader)?;
        assert_eq!(length as usize, map.len());

        for _ in 0..length {
            let key = SymbolKey::read::<R, E>(reader)?;
            let index = key as usize;
            map[index] = read_storage(key, reader)?;
        }

        Ok(Self { inner: map })
    }
}

impl BinaryWriter for SymbolStorage {
    fn write<
        W: creamy_binary::Write<Error: Send + Sync + 'static>,
        E: creamy_binary::Endianness,
    >(
        &self,
        writer: &mut W,
    ) -> Result<(), WriteError<W::Error>> {
        let length = self.inner.len() as u32;
        length.write::<W, E>(writer)?;
        for storage in &self.inner {
            let key = storage.key();
            key.write::<W, E>(writer)?;

            let mut dyn_instance = writer.as_dyn_writer();
            let mut writer = DynWriter::new(&mut dyn_instance);
            storage
                .dyn_write(E::as_dyn_endian(), &mut writer)
                .map_err(|e| match e {
                    WriteError::Custom(e) => WriteError::Custom(e),
                    WriteError::Write(e) | WriteError::Dyn(e) => WriteError::Dyn(e),
                })?;
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
