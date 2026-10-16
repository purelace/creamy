use std::assert_matches;

use creamy_binary::{BinaryReader, BinaryWriter, CustomError, ReadError};
use semver::Version;

#[test]
fn semver_serialization() -> Result<(), Box<dyn core::error::Error>> {
    let version = Version::new(0, 1, 0);
    let mut buffer = Vec::with_capacity(512);

    version.write_le(&mut buffer)?;

    // u64 + u64 + u64 + u32 + u32 = 32
    // u64 - major
    // u64 - minor
    // u64 - patch
    // u32 - prerelease string length (0)
    // u32 - build metadata string length (0)
    assert_eq!(buffer.len(), 32);

    let version_copy = Version::read_le(&mut buffer.as_slice())?;
    assert_eq!(version, version_copy);

    Ok(())
}

#[test]
fn errors() {
    let mut buffer = Vec::with_capacity(512);
    buffer.extend_from_slice(&0u64.to_le_bytes());
    buffer.extend_from_slice(&0u64.to_le_bytes());
    buffer.extend_from_slice(&7u64.to_le_bytes());

    buffer.extend_from_slice(&1u32.to_le_bytes());
    buffer.extend_from_slice(&[0xff]);

    buffer.extend_from_slice(&5u32.to_le_bytes());
    buffer.extend_from_slice("hello".as_bytes());

    let result = Version::read_le(&mut buffer.as_slice());
    assert_matches!(result, Err(ReadError::Custom(CustomError { message, .. })) if message == "error");
}

#[test]
fn error_non_ascii_prerelease() {
    let mut buffer = Vec::with_capacity(512);

    buffer.extend_from_slice(&0u64.to_ne_bytes());
    buffer.extend_from_slice(&0u64.to_ne_bytes());
    buffer.extend_from_slice(&7u64.to_ne_bytes());

    let str = "чакаво";
    buffer.extend_from_slice(&(str.len() as u32).to_ne_bytes());
    buffer.extend_from_slice(str.as_bytes());

    buffer.extend_from_slice(&0u32.to_ne_bytes());

    let result = Version::read_ne(&mut buffer.as_slice());
    assert_matches!(result, Err(ReadError::Custom(CustomError { message, .. })) if message == "version error");
}

#[test]
fn error_non_ascii_build_metadata() {
    let mut buffer = Vec::with_capacity(512);

    buffer.extend_from_slice(&0u64.to_ne_bytes());
    buffer.extend_from_slice(&0u64.to_ne_bytes());
    buffer.extend_from_slice(&7u64.to_ne_bytes());

    let str = "alpha";
    buffer.extend_from_slice(&(str.len() as u32).to_ne_bytes());
    buffer.extend_from_slice(str.as_bytes());

    let str = "чакаво";
    buffer.extend_from_slice(&(str.len() as u32).to_ne_bytes());
    buffer.extend_from_slice(str.as_bytes());

    let result = Version::read_ne(&mut buffer.as_slice());
    assert_matches!(result, Err(ReadError::Custom(CustomError { message, .. })) if message == "version error");
}
