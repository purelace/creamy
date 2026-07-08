mod error;

use std::num::NonZeroU8;

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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Core {
    path: String,
    runtime: String,
}

#[derive(BinRead, BinWrite, Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedProtocol {
    name: BString,
    group: Option<NonZeroU8>,
}

impl RequestedProtocol {
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub const fn static_group_id(&self) -> Option<NonZeroU8> {
        self.group
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedManifest {
    package: Package,
    core: Core,
    protocols: List<RequestedProtocol>,
}

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
pub struct Manifest {
    package: Package,
    protocols: List<RequestedProtocol>,
}

impl Manifest {
    /// # Errors
    ///
    /// This function will return an error if manifest has errors.
    pub fn read_manifest(manifest: &str) -> Result<(Arguments, Self), ManifestError> {
        let manifest: ParsedManifest = toml::from_str(manifest)?;

        if manifest.package.id.is_empty() {
            return Err(ManifestError::EmptyValue("id"));
        }

        if manifest.package.name.is_empty() {
            return Err(ManifestError::EmptyValue("name"));
        }

        Ok((
            Arguments {
                core: manifest.core.path,
                runtime: manifest.core.runtime,
            },
            Self {
                package: manifest.package,
                protocols: manifest.protocols,
            },
        ))
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.package.name.as_str()
    }

    #[must_use]
    pub fn requested_protocols(&self) -> &[RequestedProtocol] {
        self.protocols.as_slice()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Arguments {
    pub core: String,
    pub runtime: String,
}

/*
 * assets-*** looks for assets/
 * config-compiler looks for config/.cmyc
 * protocol-compiler looks for definitions/.xml
 * manifest validator looks for manifest.toml
 */

#[cfg(test)]
mod test {
    use std::num::NonZeroU8;

    use creamy_utils::collections::List;

    use crate::{Arguments, Manifest, Package, RequestedProtocol};

    const MANIFEST_VALID: &str = r#"
     [package]
     id = "org.creamy.test"
     name = "TestManifest"
     version = "0.4.2"
     description = "Test manifest"
     repository = "https://github.com/purelace/chocomint"
     authors = ["selrisu <myirisuchan@gmail.com>"]

     [core]
     path = "core.wasm"
     runtime = "wasm"

     [[protocols]]
     name = "testcase.valid"
     group = 1
     "#;

    #[test]
    fn valid() {
        let (arguments, manifest) = Manifest::read_manifest(MANIFEST_VALID).unwrap();

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
                protocols: List::wrap(vec![RequestedProtocol {
                    name: "testcase.valid".into(),
                    group: Some(NonZeroU8::new(1).unwrap())
                }])
            }
        );

        assert_eq!(
            arguments,
            Arguments {
                core: "core.wasm".to_string(),
                runtime: "wasm".to_string()
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

     [core]
     path = "core.wasm"
     runtime = "wasm"
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

     core.path = "core.wasm"
     core.runtime = "wasm"
     "#;

    const MANIFEST_INVALID_VERSION_MINOR: &str = r#"
      [package]
      id = "org.creamy.test"
      name = "TestManifest"
      version = "0.256.0"
      description = "Test manifest"
      repository = "https://github.com/purelace/chocomint"
      authors = ["selrisu <myirisuchan@gmail.com>"]

      core.path = "core.wasm"
      core.runtime = "wasm"
      "#;

    const MANIFEST_INVALID_VERSION_PATCH: &str = r#"
        [package]
        id = "org.creamy.test"
        name = "TestManifest"
        version = "0.0.70000"
        description = "Test manifest"
        repository = "https://github.com/purelace/chocomint"
        authors = ["selrisu <myirisuchan@gmail.com>"]

        core.path = "core.wasm"
        core.runtime = "wasm"
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
