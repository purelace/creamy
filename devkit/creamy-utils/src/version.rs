/// TODO: replace with `semver` crate
use std::{fmt::Display, str::FromStr};

use binrw::binrw;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum VersionError {
    #[error("Version must be in format MAJOR(255).MINOR(255).PATCH(65535)")]
    InvalidVersionFormat,

    #[error("Invalid major")]
    InvalidMajor,

    #[error("Invalid minor")]
    InvalidMinor,

    #[error("Invalid patch")]
    InvalidPatch,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u16,
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidVersionFormat);
        }

        Ok(Version {
            major: parts[0].parse().map_err(|_| VersionError::InvalidMajor)?,
            minor: parts[1].parse().map_err(|_| VersionError::InvalidMinor)?,
            patch: parts[2].parse().map_err(|_| VersionError::InvalidPatch)?,
        })
    }
}

pub fn deserialize_version<'de, D>(deserializer: D) -> Result<Version, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Version::from_str(&s).map_err(serde::de::Error::custom)
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
