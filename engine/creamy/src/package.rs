use hashbrown::HashMap;

use crate::{core::package::manifest::Package, error::PluginError};

pub struct PluginPackage {
    pub manifest: Package,
}

impl PluginPackage {
    pub fn from_metadata(Metadata { manifest, .. }: Metadata) -> Result<Self, PluginError> {
        let mut map = HashMap::default();
        for (name, request) in manifest.requested_protocols() {
            if let Some(index) = models.iter().enumerate().find_map(|(idx, def)| {
                if def.name().resolve(&pool) == name.as_str() {
                    Some(idx)
                } else {
                    None
                }
            }) {
                let definition = models.swap_remove(index);
                map.insert(name.as_str().into(), (definition, request.clone()));
            } else {
                return Err(PluginError::ProtocolModelNotFound {
                    target: manifest.name().into(),
                    version: request.version().clone(),
                });
            }
        }

        Ok(Self {
            manifest: manifest.into_package_manifest(),
            models: map,
        })
    }
}
