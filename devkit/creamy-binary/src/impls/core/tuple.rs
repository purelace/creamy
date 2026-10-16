#![cfg_attr(coverage_nightly, coverage(off))]

use embedded_io::{Read, Write};
use impl_trait_for_tuples::impl_for_tuples;

use crate::{BinaryReader, BinaryWriter, Endianness, ReadError, WriteError};

#[impl_for_tuples(64)]
impl BinaryWriter for Tuple {
    fn write<W: Write<Error: Send + Sync + 'static>, E: Endianness>(
        &self,
        writer: &mut W,
    ) -> Result<(), WriteError<W::Error>> {
        for_tuples!( #( Tuple.write::<W, E>(writer)?;  )* );
        Ok(())
    }
}

#[impl_for_tuples(64)]
impl BinaryReader for Tuple {
    fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        let tuple = for_tuples!( ( #( Tuple::read::<R, E>(reader)? ),* ) );
        Ok(tuple)
    }
}
