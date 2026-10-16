use crate::{BinaryReader, BinaryWriter};

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
}

#[allow(private_bounds)]
pub trait UnsignedInteger:
    core::fmt::Display
    + core::fmt::Debug
    + Clone
    + Copy
    + PartialEq
    + BinaryReader
    + BinaryWriter
    + sealed::Sealed
{
    const ZERO: Self;
    fn increment(&mut self);
    fn range_from_zero(&self) -> UnsignedRangeIterator<Self>;
}

pub struct UnsignedRangeIterator<T: UnsignedInteger> {
    current: T,
    end: T,
}

impl<T: UnsignedInteger> Iterator for UnsignedRangeIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let result = self.current;
            self.current.increment();
            Some(result)
        }
    }
}

macro_rules! impl_length {
    ($type: ident) => {
        impl UnsignedInteger for $type {
            const ZERO: Self = 0;

            fn increment(&mut self) {
                *self += 1;
            }

            fn range_from_zero(&self) -> UnsignedRangeIterator<Self> {
                UnsignedRangeIterator {
                    current: 0,
                    end: *self,
                }
            }
        }
    };
}

impl_length!(u8);
impl_length!(u16);
impl_length!(u32);
impl_length!(u64);
