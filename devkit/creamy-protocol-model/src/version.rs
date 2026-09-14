use semver::Version;
use crate::error::Fallback;

impl Fallback for Version {
    fn fallback() -> Self {
        Version::new(0, 0, 0)
    }
}

