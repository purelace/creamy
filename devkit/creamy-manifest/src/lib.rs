#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod error;

use std::collections::HashMap;

use creamy_binary::{BinRead, BinWrite};
pub use error::ManifestError;
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    id: String,
    name: String,
    version: Version,
    description: String,
    repository: String,
    authors: Vec<String>,
}

impl Package {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

//TODO: remove
impl Default for Package {
    fn default() -> Self {
        Self {
            id: String::default(),
            name: String::default(),
            version: Version::new(1, 0, 0),
            description: String::default(),
            repository: String::default(),
            authors: Vec::default(),
        }
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedProtocol {
    version: Version,
    groups: Vec<String>,
    #[serde(default)]
    provide: bool,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl RequestedProtocol {
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    #[must_use]
    pub const fn groups(&self) -> &[String] {
        self.groups.as_slice()
    }

    #[must_use]
    pub const fn provide(&self) -> bool {
        self.provide
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedManifest {
    package: Package,
    #[serde(default)]
    protocols: HashMap<String, RequestedProtocol>,
}

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
pub struct Manifest {
    package: Package,
    protocols: HashMap<String, RequestedProtocol>,
}

impl Manifest {
    #[must_use]
    pub fn into_package_manifest(self) -> Package {
        self.package
    }

    /// # Errors
    ///
    /// This function will return an error if manifest has errors.
    pub fn read_manifest(manifest: &str) -> Result<Self, ManifestError> {
        let manifest: ParsedManifest = toml::from_str(manifest)?;

        if manifest.package.id.is_empty() {
            return Err(ManifestError::EmptyValue("id"));
        }

        if manifest.package.name.is_empty() {
            return Err(ManifestError::EmptyValue("name"));
        }

        Ok(Self {
            package: manifest.package,
            protocols: manifest.protocols,
        })
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Manifest {
    #[must_use]
    pub const fn name(&self) -> &str {
        self.package.name.as_str()
    }

    #[must_use]
    pub const fn id(&self) -> &str {
        self.package.id.as_str()
    }

    #[must_use]
    pub const fn requested_groups(&self) -> &HashMap<String, RequestedProtocol> {
        &self.protocols
    }
}

#[cfg(test)]
mod test {
    use std::{assert_matches, collections::HashMap};

    use semver::Version;

    use crate::{Manifest, ManifestError, Package, RequestedProtocol};

    const MANIFEST_VALID: &str = r#"
     [package]
     id = "org.creamy.test"
     name = "TestManifest"
     version = "0.4.2"
     description = "Test manifest"
     repository = "https://github.com/purelace/chocomint"
     authors = ["selrisu <myirisuchan@gmail.com>"]

     [protocols]
     testcase = { version = "1.0.0", groups=["valid"]}
     "#;

    #[test]
    fn valid() {
        let manifest = Manifest::read_manifest(MANIFEST_VALID).unwrap();

        assert_eq!(
            manifest,
            Manifest {
                package: Package {
                    id: "org.creamy.test".into(),
                    name: "TestManifest".into(),
                    version: Version::new(0, 4, 2),
                    description: "Test manifest".into(),
                    repository: "https://github.com/purelace/chocomint".into(),
                    authors: vec!["selrisu <myirisuchan@gmail.com>".into()]
                },
                protocols: {
                    let mut map = HashMap::new();
                    map.insert(
                        "testcase".into(),
                        RequestedProtocol {
                            version: Version::new(1, 0, 0),
                            groups: vec!["valid".into()],
                            provide: false,
                        },
                    );
                    map
                }
            }
        );
    }

    const MANIFEST_INVALID_VERSION: &str = r#"
     [package]
     id = "org.creamy.test"
     name = "TestManifest"
     version = "broken"
     description = "Test manifest"
     repository = "https://github.com/purelace/chocomint"
     authors = ["selrisu <myirisuchan@gmail.com>"]
     "#;

    #[test]
    fn invalid_version_format() {
        let result = Manifest::read_manifest(MANIFEST_INVALID_VERSION);
        assert!(result.is_err());
    }

    const MANIFEST_EMPTY_ID: &str = r#"
        [package]
        id = ""
        name = "TestManifest"
        version = "0.0.70000"
        description = "Test manifest"
        repository = "https://github.com/purelace/chocomint"
        authors = ["selrisu <myirisuchan@gmail.com>"]

        "#;

    #[test]
    fn pass_empty_id() {
        let result = Manifest::read_manifest(MANIFEST_EMPTY_ID);
        assert_matches!(result, Err(ManifestError::EmptyValue("id")));
    }

    const MANIFEST_EMPTY_NAME: &str = r#"
        [package]
        id = "org.creamy.test"
        name = ""
        version = "0.0.70000"
        description = "Test manifest"
        repository = "https://github.com/purelace/chocomint"
        authors = ["selrisu <myirisuchan@gmail.com>"]

        "#;

    #[test]
    fn pass_empty_name() {
        let result = Manifest::read_manifest(MANIFEST_EMPTY_NAME);
        assert_matches!(result, Err(ManifestError::EmptyValue("name")));
    }
}
