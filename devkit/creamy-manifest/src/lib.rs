mod error;

use std::collections::HashMap;

use binrw::{BinRead, BinWrite};
use creamy_utils::{
    BString,
    collections::List,
    version::{Version, deserialize_version},
};
pub use error::ManifestError;
use serde::{Deserialize, Serialize};

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    id: BString,
    name: BString,
    #[serde(deserialize_with = "deserialize_version")]
    version: Version,
    description: BString,
    repository: BString,
    authors: List<BString>,
}

#[derive(BinRead, BinWrite, Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedProtocol {
    version: BString,
    groups: List<BString>,
    #[br(map = |val: u8| val != 0)]
    #[bw(map = |val: &bool| u8::from(*val))]
    #[serde(default)]
    provide: bool,
}

impl RequestedProtocol {
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    #[must_use]
    pub fn groups(&self) -> &[BString] {
        &self.groups
    }

    #[must_use]
    pub const fn provide(&self) -> bool {
        self.provide
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedManifest {
    package: Package,
    protocols: HashMap<BString, RequestedProtocol>,
}

#[derive(BinRead, BinWrite)]
struct MapEntry {
    key: BString,
    value: RequestedProtocol,
}

#[binrw::binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct Manifest {
    package: Package,

    #[br(temp)]
    #[bw(calc = u32::try_from(protocols.len()).unwrap())]
    entry_count: u32,

    #[br(
        count = entry_count,
        map = |vec: Vec<MapEntry>| vec.into_iter().map(|e| (e.key, e.value)).collect::<HashMap<_, _>>()
    )]
    #[bw(
        map = |map: &HashMap<BString, RequestedProtocol>| {
            map.iter().map(|(key, value)| MapEntry { key: key.clone(), value: value.clone() }).collect::<Vec<_>>()
        }
    )]
    protocols: HashMap<BString, RequestedProtocol>,
}

impl Manifest {
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

    #[must_use]
    pub fn name(&self) -> &str {
        self.package.name.as_str()
    }

    #[must_use]
    pub const fn requested_groups(&self) -> &HashMap<BString, RequestedProtocol> {
        &self.protocols
    }
}

/*
 * assets-*** looks for assets/
 * config-compiler looks for config/.cmyc
 * protocol-compiler looks for definitions/.xml
 * manifest validator looks for manifest.toml
 */

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use creamy_utils::collections::List;

    use crate::{Manifest, Package, RequestedProtocol};

    const MANIFEST_VALID: &str = r#"
     [package]
     id = "org.creamy.test"
     name = "TestManifest"
     version = "0.4.2"
     description = "Test manifest"
     repository = "https://github.com/purelace/chocomint"
     authors = ["selrisu <myirisuchan@gmail.com>"]

     [protocols]
     testcase = { version = "1.0", groups=["valid"]}
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
                    version: crate::Version {
                        major: 0,
                        minor: 4,
                        patch: 2
                    },
                    description: "Test manifest".into(),
                    repository: "https://github.com/purelace/chocomint".into(),
                    authors: List::wrap(vec!["selrisu <myirisuchan@gmail.com>".into()])
                },
                protocols: {
                    let mut map = HashMap::new();
                    map.insert(
                        "testcase".into(),
                        RequestedProtocol {
                            version: "1.0".into(),
                            groups: List::wrap(vec!["valid".into()]),
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

    const MANIFEST_INVALID_VERSION_MAJOR: &str = r#"
     [package]
     id = "org.creamy.test"
     name = "TestManifest"
     version = "1000.1.0"
     description = "Test manifest"
     repository = "https://github.com/purelace/chocomint"
     authors = ["selrisu <myirisuchan@gmail.com>"]

     "#;

    const MANIFEST_INVALID_VERSION_MINOR: &str = r#"
      [package]
      id = "org.creamy.test"
      name = "TestManifest"
      version = "0.256.0"
      description = "Test manifest"
      repository = "https://github.com/purelace/chocomint"
      authors = ["selrisu <myirisuchan@gmail.com>"]

      "#;

    const MANIFEST_INVALID_VERSION_PATCH: &str = r#"
        [package]
        id = "org.creamy.test"
        name = "TestManifest"
        version = "0.0.70000"
        description = "Test manifest"
        repository = "https://github.com/purelace/chocomint"
        authors = ["selrisu <myirisuchan@gmail.com>"]

        "#;

    #[test]
    fn invalid_version_parts() {
        let result = Manifest::read_manifest(MANIFEST_INVALID_VERSION_MAJOR);
        assert!(result.is_err());

        let result = Manifest::read_manifest(MANIFEST_INVALID_VERSION_MINOR);
        assert!(result.is_err());

        let result = Manifest::read_manifest(MANIFEST_INVALID_VERSION_PATCH);
        assert!(result.is_err());
    }
}
