use std::{collections::HashMap, num::NonZeroU8};

use creamy_cbus_driver::CreamyDriver;
use creamy_engine_core::{
    Constants, PluginLoader, WasmRuntime,
    bus::{MessageBus, config::ValidConfig},
    devkit::{
        BinaryPlugin,
        manifest::{Manifest, RequestedProtocol},
        semver::Version,
        utils::strpool::StringPool,
        xmlc::{ProtocolDefinition, StringPoolResolver},
    },
};

use crate::registry::ProtocolRegistry;

/*
* создаем карту, где название из манифеста сопоставляем с реальной моделью и мета информацией из манифеста.
* если модель не найдена, возвращаем ошибку.

* берем модели из карты и проверяем какие надо задекларировать
* если модель нужно задекларировать, то проверяем, есть ли уже поставщики,
* если есть то ошибка, иначе добавляем.
*
* после этого сверяем запрашиваемые модели.
* если модели различаются, выкидываем ошибку.
*
*/

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Protocol model `{target_model}@{version}` not found")]
    ProtocolModelNotFound {
        target_model: Box<str>,
        version: Version,
    },
    #[error("Protocol `{target_model}@{version}` has already been declared")]
    ProtocolDeclaredAlready {
        target_model: Box<str>,
        version: Version,
    },
    #[error(
        "Protocols `{target_model}@{version_a}` and `{target_model}@{version_b}` have different models"
    )]
    DifferentProtocolModels {
        target_model: Box<str>,
        version_a: Version,
        version_b: Version,
    },
}

struct TempPluginPackage {
    version: Version,
    manifest: Manifest,
    pool: StringPool,
    definitions: HashMap<Box<str>, (ProtocolDefinition, RequestedProtocol)>,
}

impl TempPluginPackage {
    fn from_package(
        BinaryPlugin {
            version,
            manifest,
            pool,
            mut definitions,
            ..
        }: BinaryPlugin,
    ) -> Result<Self, Error> {
        let mut map = HashMap::new();
        for (name, request) in manifest.requested_groups() {
            if let Some(index) = definitions.iter().enumerate().find_map(|(idx, def)| {
                if def.name().resolve(&pool) == name.as_str() {
                    Some(idx)
                } else {
                    None
                }
            }) {
                let definition = definitions.swap_remove(index);
                map.insert(name.as_str().into(), (definition, request.clone()));
            } else {
                return Err(Error::ProtocolModelNotFound {
                    target_model: manifest.name().into(),
                    version: request.version().clone(),
                });
            }
        }

        Ok(Self {
            version,
            manifest,
            pool,
            definitions: map,
        })
    }
}

pub struct PluginEngine<R: WasmRuntime, L: PluginLoader> {
    bus: MessageBus<CreamyDriver, R::Module>,
    constants: ValidConfig<Constants>,
    runtime: R,
    loader: L,
    registry: ProtocolRegistry,
}

impl<R: WasmRuntime, L: PluginLoader> PluginEngine<R, L> {
    pub fn new(constants: ValidConfig<Constants>, runtime: R, loader: L) -> Self {
        Self {
            bus: MessageBus::new(&constants, CreamyDriver::new),
            constants,
            runtime,
            loader,
            registry: ProtocolRegistry::default(),
        }
    }

    fn init_package(&mut self, package: BinaryPlugin) -> Result<(), Error> {
        let module = self
            .runtime
            .init_module(&self.constants, package.core())
            .unwrap();
        self.bus.add_subscriber(|_, _| module).unwrap();

        //let path = format!("{}@{}", package.manifest().name(), package.version());

        let package = TempPluginPackage::from_package(package)?;
        let protocol_name = package.manifest.name();

        for (name, (mut def, request)) in package.definitions {
            self.registry.replace_strings(&package.pool, &mut def);
            if request.provide() {
                match self.registry.get_model(&name) {
                    Some(def) => {
                        return Err(Error::ProtocolDeclaredAlready {
                            target_model: name,
                            version: def.version().clone(),
                        });
                    }
                    None => {
                        self.registry.declare_protocol(protocol_name, def);
                    }
                }
            } else {
                match self.registry.get_model(&name) {
                    Some(decl) => {
                        if decl != &def {
                            return Err(Error::DifferentProtocolModels {
                                target_model: name,
                                version_a: def.version().clone(),
                                version_b: decl.version().clone(),
                            });
                        }
                        //TODO: provided
                    }
                    None => {
                        return Err(Error::ProtocolModelNotFound {
                            target_model: name,
                            version: def.version().clone(),
                        });
                    }
                }
            }
        }

        Ok(())
        //let driver = self.bus.get_driver_mut();
    }

    pub fn tick(&mut self, roundtrip: NonZeroU8) {
        self.loader.load();

        while self.loader.loaded() != 0
            && let Some(package) = self.loader.take_loaded_package()
        {
            if let Err(e) = self.init_package(package) {
                tracing::error!("{e}");
            }
        }

        for _ in 0..roundtrip.get() {
            self.bus.tick();
        }
    }

    #[must_use]
    pub const fn loaded_plugins(&self) -> u8 {
        self.bus.subscribers()
    }

    pub const fn protocol_registry(&self) -> &ProtocolRegistry {
        &self.registry
    }
}
