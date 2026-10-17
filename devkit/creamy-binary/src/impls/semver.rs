use alloc::boxed::Box;

use embedded_io::{Read, Write};
use semver::{BuildMetadata, Prerelease, Version};
use smol_str::SmolStr;

use crate::{BinaryReader, BinaryWriter, CustomError, Endianness, ReadError, WriteError};

struct VersionError(semver::Error);

impl core::fmt::Debug for VersionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl core::fmt::Display for VersionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl core::error::Error for VersionError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &'static str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn core::error::Error> {
        self.source()
    }
}

impl BinaryReader for Version {
    fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        let major = u64::read::<R, E>(reader)?;
        let minor = u64::read::<R, E>(reader)?;
        let patch = u64::read::<R, E>(reader)?;
        let pre = SmolStr::read::<R, E>(reader)?;
        let build = SmolStr::read::<R, E>(reader)?;

        Ok(Self {
            major,
            minor,
            patch,
            pre: Prerelease::new(pre.as_str()).map_err(|e| {
                ReadError::Custom(CustomError {
                    message: "version error".into(),
                    error: Some(Box::new(VersionError(e))),
                })
            })?,
            build: BuildMetadata::new(build.as_str()).map_err(|e| {
                ReadError::Custom(CustomError {
                    message: "version error".into(),
                    error: Some(Box::new(VersionError(e))),
                })
            })?,
        })
    }
}

impl BinaryWriter for Version {
    fn write<W: Write<Error: Send + Sync + 'static>, E: Endianness>(
        &self,
        writer: &mut W,
    ) -> Result<(), WriteError<W::Error>> {
        self.major.write::<W, E>(writer)?;
        self.minor.write::<W, E>(writer)?;
        self.patch.write::<W, E>(writer)?;
        self.pre.as_str().write::<W, E>(writer)?;
        self.build.as_str().write::<W, E>(writer)?;
        Ok(())
    }
}
