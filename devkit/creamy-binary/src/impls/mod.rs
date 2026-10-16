mod alloc;
mod core;

#[cfg(feature = "smol_str")]
mod smol_str;

#[cfg(feature = "semver")]
mod semver;

#[cfg(feature = "hash")]
mod hash;

#[cfg(feature = "std")]
mod hash_std;
