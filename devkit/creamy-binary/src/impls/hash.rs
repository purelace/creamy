use core::hash::{BuildHasher, Hash};

use hashbrown::HashMap;

use crate::{BinaryReader, BinaryWriter};

impl<K, V, S> BinaryReader for HashMap<K, V, S>
where
    K: BinaryReader + Eq + Hash,
    V: BinaryReader,
    S: BuildHasher + Default,
{
    fn read<R: embedded_io::Read, E: crate::Endianness>(
        reader: &mut R,
    ) -> Result<Self, crate::ReadError<R::Error>> {
        let len = u32::read::<R, E>(reader)?;
        let mut map = HashMap::with_capacity_and_hasher(len as usize, S::default());

        for _ in 0..len {
            let key = K::read::<R, E>(reader)?;
            let value = V::read::<R, E>(reader)?;
            map.insert(key, value);
        }

        Ok(map)
    }
}

impl<K, V, S> BinaryWriter for HashMap<K, V, S>
where
    K: BinaryWriter,
    V: BinaryWriter,
{
    fn write<W: embedded_io::Write<Error: Send + Sync + 'static>, E: crate::Endianness>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::WriteError<W::Error>> {
        (self.len() as u32).write::<W, E>(writer)?;
        for (k, v) in self {
            k.write::<W, E>(writer)?;
            v.write::<W, E>(writer)?;
        }

        Ok(())
    }
}
