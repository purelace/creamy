use alloc::boxed::Box;

use creamy_engine_core::Metadata;
use hashbrown::HashMap;
use rustc_hash::FxBuildHasher;

use crate::{
    core::package::{
        manifest::{Package, RequestedProtocol},
        model::definition::ProtocolModel,
        pool::{StringPool, StringPoolResolver},
    },
    error::PluginError,
};

pub type Models = HashMap<Box<str>, (ProtocolModel, RequestedProtocol), FxBuildHasher>;

pub struct PluginPackage {
    pub manifest: Package,
    pub pool: StringPool,
    pub models: Models,
}

impl PluginPackage {
    pub fn from_metadata(
        Metadata {
            manifest,
            pool,
            mut models,
            ..
        }: Metadata,
    ) -> Result<Self, PluginError> {
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
                    target_model: manifest.name().into(),
                    version: request.version().clone(),
                });
            }
        }

        Ok(Self {
            manifest: manifest.into_package_manifest(),
            pool,
            models: map,
        })
    }
}
